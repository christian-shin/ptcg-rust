//! The damage events of events batch 6 (docs/design/events-design.md, sections 4 and 4.1): PlaceCounters (and, as they
//! land, MoveCounters and Damage).
//!
//! Every path that puts damage counters on a Pokémon calls its routine here, in the `engine::enter` / `engine::attach` /
//! `engine::condition` pattern: the routine checks the event (a Pokémon is there; the locks, `derived::event_locked`;
//! the preventions, `derived::event_prevented`), produces it (an `Effect` dispatched to the cards), applies its
//! consequences in [`reducer`], and the triggers over it run after. A refused event doesn't happen (`Ok(false)`) and
//! the op it belongs to continues (id2264, id2425: Dusknoir's Cursed Blast still Knocks Out Dusknoir when Battle Cage or
//! Hide 'n' Sneak stops the counters).
//!
//! - [`place`]: "put N damage counters on ..." whatever causes it: an attack's effect, an Ability, a Trainer, a
//!   Stadium (Risky Ruins), a Tool (Punk Helmet), an Energy (Spiky Energy), a Special Condition's rule (Poison and
//!   Burn at the Checkup, Confusion). Damage counters are not damage: no Weakness or Resistance, no "damaged" triggers,
//!   no damage-taken record (APR C-07). The Knock Out waits for the state check after the whole action (id66; all
//!   counters are placed first, id1547).
//!
//! - [`deal`]: an attack's damage to a Pokémon (APR A-01, B-01..B-09, Damage Calculation Order 1-6): the calculation
//!   (steps 2-5 as today's passes, `DealDamage` / `ApplyWeakness` / `PutDamage`: B6-OLD -> batch 8, the derived layer),
//!   then step 6, the preventions of damage (`Prevent` naming `Kind(Damage)`: "prevent all damage done to ..."; APR
//!   C-16: the damage becomes 0 and the calculation ends), skipped when the attack ignores the effects on the damaged
//!   Pokémon (Shred), then the Damage event: the counters, the damage-taken records, and the "is damaged by an attack"
//!   triggers recorded for step 6 of the attack (Spiky Energy, Punk Helmet, the delayed traps). Damage is not an effect:
//!   "prevent all effects of attacks" never stops it (APR C-17), and the records of what an attack damaged (what
//!   "Knocked Out by damage from an attack" reads) are written whatever protects the Pokémon from effects.
//! - [`move_counters`]: "move N damage counters from ... to ..." (events design 4.1; APR C-08): one event per action,
//!   carrying its pairs. A lock forbids the whole action (Patrat's Watchful Eye: the counters stay, id2350); per pair,
//!   a protected source keeps its counters (nothing moves, id390, id2150, id2192), a protected destination makes the
//!   counters removed from the source vanish (id2257, id79, id393, id1876; JP FAQ Battle Cage x3). Moving counters
//!   isn't healing and isn't damage (id1995, id2254, id554; id251, id1650). The Knock Outs wait for the state check
//!   after the whole action (id66; a one-at-a-time Ability's use is its own action, id67, id2152).
//!
//! The Cause is the frame's (`spec::run::Frame::cause`) or the rule's; ops never pick one.

use crate::cause::Cause;
use crate::effects::{AtkBase, CounterMove, EffId, Effect, SlotRef};
use crate::list::SVec;
use crate::game::{Game, R};
use crate::spec::event::*;

// ---------------------------------------------------------------------------
// Event views

/// The PlaceCounters event: `amount` HP of damage counters on the Pokémon in `target`.
pub fn counters_view(g: &Game, target: SlotRef, amount: i32, cause: Cause) -> EventView {
    EventView { card: g.st.slot_pokemon(target.p as usize, target.s), slot: Some(target), amount, ..EventView::new(EventKind::PlaceCounters, cause, target.p, crate::spec::event::whose_turn(g)) }
}

/// The MoveCounters event as one end of a move sees it: `amount` HP of counters leaving (`From`) or arriving on (`To`)
/// the Pokémon in `slot` (events design 4.1: a protected source keeps its counters, a protected destination makes them
/// vanish).
pub fn move_view(g: &Game, slot: SlotRef, end: MoveEnd, amount: i32, cause: Cause) -> EventView {
    EventView { card: g.st.slot_pokemon(slot.p as usize, slot.s), slot: Some(slot), amount, end: Some(end), ..EventView::new(EventKind::MoveCounters, cause, slot.p, crate::spec::event::whose_turn(g)) }
}

/// The Damage event: `amount` of the attack's damage to the Pokémon in `b.target`, by the attack (`b.cause`).
pub fn damage_view(g: &Game, b: &AtkBase, amount: i32) -> EventView {
    let t = b.target;
    EventView {
        card: g.st.slot_pokemon(t.p as usize, t.s),
        slot: Some(t),
        amount,
        ignores_defender: crate::prefabs::ignores_defender_effects(g, b),
        ..EventView::new(EventKind::Damage, b.cause, t.p, crate::spec::event::whose_turn(g))
    }
}

// ---------------------------------------------------------------------------
// The routines

/// Damage: the attack of `b` does `amount` damage to the Pokémon in `b.target`. `deal`: the damage to the Defending
/// Pokémon (the `DealDamage` route: the attacker-side modifiers, Weakness and Resistance); otherwise put on the Pokémon
/// (`PutDamage`: Benched Pokémon; Weakness and Resistance still for the opponent's Active Pokémon, as today). The
/// calculation passes keep today's order exactly (B6-OLD -> batch 8). Nothing happens when the final damage is 0 or a
/// prevention applies.
pub fn deal(g: &mut Game, b: AtkBase, amount: i32, deal: bool) -> R {
    let t = b.target;
    let mut d = amount;
    let mut weakness_applied = false;
    let (ig_w, ig_r) = match *g.e(b.attack_effect) {
        Effect::Attack { ignore_weakness, ignore_resistance, .. } => (ignore_weakness, ignore_resistance),
        _ => (false, false),
    };
    if deal {
        // Step 2: the attacker-side modifiers (DamageDealt passives), then "the Defending Pokémon's attacks do N less".
        let (e, _) = g.run_fx(Effect::DealDamage { b, damage: d })?;
        if let Effect::DealDamage { damage, .. } = e {
            d = damage;
        }
        let src_red = g.st.slot(b.source.p as usize, b.source.s).attack_damage_reduction_next_turn;
        if src_red > 0 {
            d = (d - src_red).max(0);
        }
        // Steps 3-4: Weakness and Resistance apply to the opponent's Active Pokémon only (APR B-08: not to the attack's
        // damage to the attacker itself).
        let opp = 1 - b.player as usize;
        if t.p as usize == opp && t.s == g.st.players[opp].active {
            let (e, _) = g.run_fx(Effect::ApplyWeakness { b, damage: d, ignore_weakness: ig_w, ignore_resistance: ig_r })?;
            if let Effect::ApplyWeakness { damage, .. } = e {
                d = damage;
            }
        }
        weakness_applied = true;
    }
    // Step 5: the defender-side modifiers (DamageTaken passives), and the survive-on-10 replacements of a full-HP Pokémon.
    let (e, _) = g.run_fx(Effect::PutDamage { b, damage: d, weakness_applied, survive_on_ten_hp: false })?;
    let survive = match e {
        Effect::PutDamage { damage, survive_on_ten_hp, .. } => {
            d = damage;
            survive_on_ten_hp
        }
        _ => false,
    };
    if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
        crate::bail!("ILLEGAL_ACTION");
    }
    let opp = 1 - b.player as usize;
    let shred = crate::prefabs::ignores_defender_effects(g, &b);
    if !weakness_applied {
        let src_red = g.st.slot(b.source.p as usize, b.source.s).attack_damage_reduction_next_turn;
        if src_red > 0 {
            d = (d - src_red).max(0);
        }
        if t.p as usize == opp && t.s == g.st.players[opp].active {
            let (e, _) = g.run_fx(Effect::ApplyWeakness { b, damage: d, ignore_weakness: ig_w, ignore_resistance: ig_r })?;
            if let Effect::ApplyWeakness { damage, .. } = e {
                d = damage;
            }
        }
    }
    // The lasting defender-side effects: "takes N less damage", "the Defending Pokémon takes N more damage" (summed,
    // floored once: APR B-05).
    if !shred {
        let ts = g.st.slot(t.p as usize, t.s);
        d -= ts.damage_reduction_next_turn;
        if ts.defending_extra_damage_next_turn > 0 && !ts.defending_extra_damage_pending && ts.defending_extra_damage_attacker == Some(b.player) {
            d += ts.defending_extra_damage_next_turn;
        }
    }
    d = d.max(0);
    // Step 6: "prevent all damage done to ..." (in play, and the lasting ones the Pokémon's own attack left), then the
    // coin-flip preventions, flipped only for damage no other prevention stops (user decision D8). Skipped when the attack
    // isn't affected by effects on the damaged Pokémon (Shred; APR C-16).
    if !shred {
        let v = damage_view(g, &b, d);
        if crate::derived::event_prevented(g, &v)? {
            return Ok(());
        }
        if d > 0 && crate::spec::passive::coin_prevented(g, &v)? {
            return Ok(());
        }
    }
    if d <= 0 {
        return Ok(());
    }
    g.run_fx_unit(Effect::Damage { b, amount: d, survive })
}

/// Would the attack's damage `amount` to `b.target` be prevented now (the step 6 preventions without the coin flips)? A
/// Tool that is discarded after it reduced the damage is kept when the damage is prevented.
pub fn would_be_prevented(g: &mut Game, b: &AtkBase, amount: i32) -> R<bool> {
    let v = damage_view(g, b, amount);
    if v.ignores_defender {
        return Ok(false);
    }
    crate::derived::event_prevented(g, &v)
}

/// PlaceCounters: `hp` HP of damage counters on the Pokémon in `target`. It doesn't happen (`Ok(false)`) when no
/// Pokémon is there, there is nothing to place, or the event is refused (a lock; a prevention: Hide 'n' Sneak, Mist
/// Energy, Battle Cage, ...).
pub fn place(g: &mut Game, target: SlotRef, hp: i32, cause: Cause) -> R<bool> {
    if hp <= 0 || g.st.slot_pokemon(target.p as usize, target.s).is_none() {
        return Ok(false);
    }
    let v = counters_view(g, target, hp, cause);
    if crate::engine::condition::refused(g, &v)?.is_some() {
        return Ok(false);
    }
    g.run_fx_unit(Effect::PlaceCounters { p: target.p, target, amount: hp, cause })?;
    Ok(true)
}

/// MoveCounters: one action moving counters, `pairs` of (from, to, HP) in the order the text or the player gives them.
/// Each pair moves as much as it can (the counters its source has then, id63). Returns what happened to each pair
/// (removed from the source, placed on the destination); nothing moves when a lock forbids the action.
pub fn move_counters(g: &mut Game, pairs: &[(SlotRef, SlotRef, i32)], cause: Cause) -> R<SVec<CounterMove, 16>> {
    let mut moves: SVec<CounterMove, 16> = SVec::new();
    let Some(&(first, _, _)) = pairs.first() else { return Ok(moves) };
    // The locks forbid the action (Patrat's Watchful Eye binds both players), asked once.
    let v = move_view(g, first, MoveEnd::From, pairs.iter().map(|x| x.2).sum(), cause);
    if crate::derived::event_locked(g, &v)?.is_some() {
        return Ok(moves);
    }
    // The damage each spot has as the pairs are carried out in order.
    let mut left: SVec<(SlotRef, i32), 16> = SVec::new();
    let damage_of = |g: &Game, left: &SVec<(SlotRef, i32), 16>, s: SlotRef| left.iter().find(|(x, _)| *x == s).map(|(_, d)| *d).unwrap_or(g.st.slot(s.p as usize, s.s).damage);
    let set = |left: &mut SVec<(SlotRef, i32), 16>, s: SlotRef, d: i32| match left.as_mut_slice().iter_mut().find(|(x, _)| *x == s) {
        Some(e) => e.1 = d,
        None => left.push((s, d)),
    };
    for &(from, to, hp) in pairs {
        if g.st.slot_pokemon(from.p as usize, from.s).is_none() || g.st.slot_pokemon(to.p as usize, to.s).is_none() {
            continue;
        }
        let removed = hp.min(damage_of(g, &left, from));
        if removed <= 0 {
            continue;
        }
        let vf = move_view(g, from, MoveEnd::From, removed, cause);
        if crate::derived::event_prevented(g, &vf)? {
            continue;
        }
        let vt = move_view(g, to, MoveEnd::To, removed, cause);
        let placed = if crate::derived::event_prevented(g, &vt)? { 0 } else { removed };
        let d = damage_of(g, &left, from);
        set(&mut left, from, d - removed);
        let d = damage_of(g, &left, to);
        set(&mut left, to, d + placed);
        moves.push(CounterMove { from, to, removed, placed });
    }
    if !moves.is_empty() {
        g.run_fx_unit(Effect::MoveCounters { p: cause.player, moves, cause })?;
    }
    Ok(moves)
}

// ---------------------------------------------------------------------------
// Consequences, applied by the events' reducer

/// The reducer of this module's events: what each does to the game. Moving counters off a Pokémon isn't healing it
/// ("healed this turn" untouched; id1995, id2254).
pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::PlaceCounters { target, amount, .. } => {
            g.st.players[target.p as usize].slots[target.s as usize].damage += amount.max(0);
        }
        Effect::Damage { b, amount, survive } => {
            let t = b.target;
            let (tp, ts) = (t.p as usize, t.s);
            let Some(card) = g.st.slot_pokemon(tp, ts) else { crate::bail!("ILLEGAL_ACTION") };
            g.st.players[tp].slots[ts as usize].damage += amount;
            g.st.cards[card as usize].damage_taken_last_turn += amount;
            // The survive-on-10 replacement a full-HP Pokémon's effect armed in the calculation (its remaining HP is 10).
            if survive {
                g.st.players[tp].slots[ts as usize].hp_bonus = 0;
                g.run_fx_unit(Effect::CheckHp { p: b.player, target: t, card: Some(card) })?;
                let hp = crate::engine::check::hp_of(g, tp, ts, Some(card));
                if g.st.slot(tp, ts).damage >= hp {
                    g.st.players[tp].slots[ts as usize].damage = hp - 10;
                    if !g.ten_hp.contains(&t) {
                        g.ten_hp.push(t);
                    }
                }
            }
            // What the attack damaged, for "Knocked Out by damage from an attack" (the opponent's Pokémon, any spot; and
            // those it damaged in the Active Spot). Damage is not an effect: written whatever protects the Pokémon.
            if t.p != b.player && g.st.phase == crate::types::GamePhase::Attack {
                if let Some(la) = g.last_attack.as_mut() {
                    if !la.damaged.contains(&t) {
                        la.damaged.push(t);
                    }
                    if g.st.players[tp].active == ts && !la.damaged_active.contains(&t) {
                        la.damaged_active.push(t);
                    }
                }
                // A delayed trap on the damaged Pokémon ("if this Pokémon is damaged by an attack during your opponent's
                // next turn, put N damage counters on the Attacking Pokémon"): recorded for step 6 of the attack.
                let slot = g.st.slot(tp, ts);
                let armed = if slot.retaliate_on_damage_next_turn_pending.is_some() { None } else { slot.retaliate_on_damage_next_turn };
                if let Some(r) = armed {
                    if r.damage > 0 {
                        g.attack_trigger(b, amount, r.source_card, Some(r), false)?;
                    }
                }
            }
        }
        Effect::MoveCounters { moves, .. } => {
            for m in moves.iter() {
                let s = &mut g.st.players[m.from.p as usize].slots[m.from.s as usize];
                s.damage = (s.damage - m.removed).max(0);
                g.st.players[m.to.p as usize].slots[m.to.s as usize].damage += m.placed;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! The routines against hand-built boards.
    use super::*;
    use crate::types::GamePhase;
    use serde_json::json;

    const MILOTIC: &str = "Milotic ex SSP 42";
    const OGERPON: &str = "Teal Mask Ogerpon ex TWM 25";
    const SNORLAX: &str = "Hop's Snorlax JTG 117";
    const MIST: &str = "Mist Energy TEF 161";

    fn game(sc: serde_json::Value) -> Game {
        let mut names: Vec<&str> = Vec::new();
        for n in [MILOTIC, OGERPON, SNORLAX, MIST] {
            names.extend(std::iter::repeat(n).take(4));
        }
        while names.len() < 60 {
            names.push("Grass Energy MEE 1");
        }
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    /// An attack of `p`'s Active Pokémon (its first attack) in progress, against `target`.
    fn attack_on(g: &mut Game, p: usize, target: SlotRef) -> AtkBase {
        let source = SlotRef::new(p, g.st.players[p].active);
        let card = g.st.slot_pokemon(p, source.s).unwrap();
        let attack = crate::state::AttackRef { card, index: 0 };
        let atk = g.new_fx(Effect::Attack { p: p as u8, opp: (1 - p) as u8, attack, damage: 0, ignore_weakness: true, ignore_resistance: true, ignore_defender_effects: false, source, barrage_used: false });
        g.st.phase = GamePhase::Attack;
        g.last_attack = Some(crate::game::LastAttack { p: p as u8, effect: atk, attack, source, pokemon: Some(card), damaged_active: SVec::new(), damaged: SVec::new() });
        AtkBase { attack_effect: atk, player: p as u8, opponent: (1 - p) as u8, attack, source, target, cause: Cause::of_attack_at(g, p as u8, attack, source) }
    }

    /// "Prevent all damage" is step 6 of the calculation (APR C-16): the damage becomes 0 and the calculation ends, so a
    /// "takes N more damage" effect on the Pokémon (step 5) can't bring it back (Milotic ex's Sparkling Scales against a
    /// Tera Pokémon's attack).
    #[test]
    fn prevent_all_damage_is_step_6() {
        let mut g = game(json!({"me": {"reset": true, "active": OGERPON}, "opp": {"reset": true, "active": MILOTIC}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let t = SlotRef::new(o, g.st.players[o].active);
        {
            let s = &mut g.st.players[o].slots[t.s as usize];
            s.defending_extra_damage_next_turn = 30;
            s.defending_extra_damage_attacker = Some(me as u8);
        }
        let b = attack_on(&mut g, me, t);
        deal(&mut g, b, 100, true).unwrap();
        assert_eq!(g.st.slot(o, t.s).damage, 0, "Sparkling Scales prevents the Tera Pokémon's damage, the 30 more included");
        // A non-Tera attacker's damage (Milotic ex's own) gets the 30 more.
        let mut g = game(json!({"me": {"reset": true, "active": MILOTIC}, "opp": {"reset": true, "active": MILOTIC}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let t = SlotRef::new(o, g.st.players[o].active);
        {
            let s = &mut g.st.players[o].slots[t.s as usize];
            s.defending_extra_damage_next_turn = 30;
            s.defending_extra_damage_attacker = Some(me as u8);
        }
        let b = attack_on(&mut g, me, t);
        deal(&mut g, b, 100, true).unwrap();
        assert_eq!(g.st.slot(o, t.s).damage, 130);
    }

    /// Damage is not an effect (APR C-17): Mist Energy doesn't stop it, and the attack's record of what it damaged ("Knocked
    /// Out by damage from an attack") is written.
    #[test]
    fn damage_is_not_an_effect() {
        let mut g = game(json!({"me": {"reset": true, "active": MILOTIC}, "opp": {"reset": true, "active": SNORLAX, "active_energy": [MIST]}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let t = SlotRef::new(o, g.st.players[o].active);
        let b = attack_on(&mut g, me, t);
        deal(&mut g, b, 60, true).unwrap();
        assert_eq!(g.st.slot(o, t.s).damage, 60);
        let la = g.last_attack.unwrap();
        assert!(la.damaged.contains(&t) && la.damaged_active.contains(&t));
        // Its counters are an effect: prevented.
        assert!(!place(&mut g, t, 20, b.cause).unwrap());
        assert_eq!(g.st.slot(o, t.s).damage, 60);
    }

    /// Moving counters: a protected destination makes them vanish, a protected source keeps them (events design 4.1).
    #[test]
    fn move_counters_ends() {
        let mut g = game(json!({"me": {"reset": true, "active": SNORLAX}, "opp": {"reset": true, "active": SNORLAX, "active_damage": 30, "bench": [{"card": SNORLAX, "damage": 30, "energy": [MIST]}, {"card": SNORLAX}]}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let act = SlotRef::new(o, g.st.players[o].active);
        let misty = SlotRef::new(o, g.st.players[o].bench.as_slice()[0]);
        let plain = SlotRef::new(o, g.st.players[o].bench.as_slice()[1]);
        let cause = attack_on(&mut g, me, act).cause;
        let m = move_counters(&mut g, &[(act, misty, 20), (misty, plain, 10)], cause).unwrap();
        assert_eq!(m.as_slice(), &[CounterMove { from: act, to: misty, removed: 20, placed: 0 }]);
        assert_eq!((g.st.slot(o, act.s).damage, g.st.slot(o, misty.s).damage, g.st.slot(o, plain.s).damage), (10, 30, 0));
        assert!(!g.st.slot(o, act.s).healed_this_turn, "moving counters isn't healing");
    }
}
