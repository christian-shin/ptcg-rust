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
//!   (step 2 the attacker-side declarations, steps 3-4 Weakness and Resistance from the derived layer, step 5 the
//!   defender-side declarations and survive-on-10 replacements; batch 8 C4, no dispatched pass),
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
use crate::list::{CardList, SVec};
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
        // Done to a Pokémon in the Active Spot (written by the reducer: read after the damage, id1992).
        damaged_active: g.last_attack.as_ref().map_or(false, |la| la.damaged_active.contains(&t)),
        ..EventView::new(EventKind::Damage, b.cause, t.p, crate::spec::event::whose_turn(g))
    }
}

// ---------------------------------------------------------------------------
// The routines

/// Damage: the attack of `b` does `amount` damage to the Pokémon in `b.target`, in one calculation order (APR Damage
/// Calculation Order 1-6). The attacker-side modifiers (step 2) apply to every damage the attack does; `deal` adds
/// Weakness and Resistance when the target is in an Active Spot (steps 3-4); otherwise the damage is put on the Pokémon
/// (`DamageCalc::Put`: no Weakness or Resistance). Nothing happens when the final damage is 0 or a prevention applies (the
/// record shows a damage of 0, `record::comes_out_zero`).
pub fn deal(g: &mut Game, b: AtkBase, amount: i32, deal: bool) -> R {
    let t = b.target;
    let mut d = amount;
    let (ig_w, ig_r) = match *g.e(b.attack_effect) {
        Effect::Attack { ignore_weakness, ignore_resistance, .. } => (ignore_weakness, ignore_resistance),
        _ => (false, false),
    };
    // Step 2: the effects on the Pokémon doing the damage: the attacker-side modifiers (DamageDealt at the Deal stage), then
    // "the Defending Pokémon's attacks do N less damage".
    // Every damage the attack does, to a Benched Pokémon and to the attacker itself too (id158, id2213, id2345: "attacks used
    // by your opponent's Active Pokémon do N less damage" reduces the damage to each Pokémon): a modifier names the Pokémon
    // it is for (`DamageDealtSpec::target`; the default is the opponent's Active Pokémon, which a Benched one never is).
    {
        let mut acc = crate::spec::passive::DamageAcc { b, damage: d, weakness_applied: true, survive: false, flags: 0, applied: crate::list::SVec::new() };
        attacker_side(g, &mut acc)?;
        d = acc.damage;
    }
    let src_red = crate::engine::lasting::deals_less_damage(&g.st, b.source.p as usize, b.source.s);
    if src_red > 0 {
        d -= src_red;
    }
    // APR "Damage Calculation Order", step 2: an effect that makes the damage 0 or negative ends the calculation.
    if d <= 0 {
        crate::record::comes_out_zero(g, EventKind::Damage, t, b.cause);
        return Ok(());
    }
    // Steps 3-4: Weakness and Resistance apply to damage done to a Pokémon in an Active Spot, either player's: the
    // attack's damage to the attacker itself too (APR B-09: "Weakness, Resistance, and other effects on the Pokémon
    // still apply"); never to a Benched Pokémon unless the text says so (APR B-08, B-09).
    if deal && t.s == g.st.players[t.p as usize].active {
        d = crate::engine::game_effect::weakness_and_resistance(g, &b, d, ig_w, ig_r)?;
    }
    // Step 5: the effects on the damaged Pokémon: the defender-side modifiers (DamageTaken), the survive-on-10
    // replacements of a full-HP Pokémon (SurviveOnTen), then the lasting "takes N less / more damage".
    let mut acc = crate::spec::passive::DamageAcc { b, damage: d, weakness_applied: deal, survive: false, flags: 0, applied: crate::list::SVec::new() };
    defender_side(g, &mut acc)?;
    d = acc.damage;
    let survive = acc.survive;
    if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
        crate::bail!("ILLEGAL_ACTION");
    }
    let shred = crate::prefabs::ignores_defender_effects(g, &b);
    // The lasting defender-side effects: "takes N less damage", "the Defending Pokémon takes N more damage" (summed,
    // floored once: APR B-05).
    if !shred {
        d -= crate::engine::lasting::takes_less_damage(g, &b)?;
        d += crate::engine::lasting::takes_more_damage(&g.st, t.p as usize, t.s, b.player);
    }
    d = d.max(0);
    // Step 6: "prevent all damage done to ..." (in play, and the lasting ones the Pokémon's own attack left), then the
    // coin-flip preventions, flipped only for damage no other prevention stops (user decision D8). Skipped when the attack
    // isn't affected by effects on the damaged Pokémon (Shred; APR C-16).
    if !shred {
        let v = damage_view(g, &b, d);
        if crate::derived::event_prevented(g, &v)? || (d > 0 && crate::spec::passive::coin_prevented(g, &v)?) {
            d = 0;
        }
    }
    if d <= 0 {
        crate::record::comes_out_zero(g, EventKind::Damage, t, b.cause);
        return Ok(());
    }
    g.run_fx_unit(Effect::Damage { b, amount: d, survive })
}

/// The base damage of the attack of `b` came out 0 (APR A-01 step 4; Damage Calculation Order 1): no calculation is done.
/// A damage that was calculated and came out 0 (a multiplier with no count: "30x" with no heads) shows a damage of 0 to the
/// Defending Pokémon in the record (`record::comes_out_zero`). An attack whose text made it do nothing
/// (`fx_flag::DOES_NOTHING`) and an attack without printed damage do no damage at all: no entry.
pub fn base_damage_zero(g: &mut Game, b: &AtkBase) {
    if g.record.on() && g.fx_flags(b.attack_effect) & crate::game::fx_flag::DOES_NOTHING == 0 && crate::engine::attack::attack_def(g, b.attack).damage > 0 {
        crate::record::comes_out_zero(g, EventKind::Damage, b.target, b.cause);
    }
}

/// Step 2's declarations: every card of the game with an attacker-side modifier at the Deal stage (`decl::fam::DAMAGE_DEALT`,
/// any zone: each checks where its card is), in the propagation order, run undelegated as the card handlers were; then a
/// copied attack's source card run for the copycat (`copy_attack::delegate_damage`).
fn attacker_side(g: &mut Game, acc: &mut crate::spec::passive::DamageAcc) -> R {
    use crate::spec::passive::Modifier;
    let order = g.decl_all(crate::decl::fam::DAMAGE_DEALT, 0);
    if !order.is_empty() || crate::engine::lasting::has_declared(&g.st) {
        crate::derived::Facts::undelegated(g, |g| -> R {
            for c in order.iter().copied() {
                for ps in crate::decl::passives(g, c) {
                    if let Modifier::DamageDealt(d) = &ps.modifier {
                        crate::spec::passive::damage_dealt_deal(g, c, ps.origin, d, acc)?;
                    }
                }
            }
            // The effects cards put for a time ("during this turn, ..."), after the cards' own declarations.
            crate::spec::passive::declared_damage_dealt(g, acc)
        })?;
    }
    crate::copy_attack::delegate_damage(g, acc, false)
}

/// Step 5's declarations: every card with a defender-side modifier or a survive-on-10 replacement
/// (`decl::fam::DAMAGE_TAKEN`, any zone), in the propagation order, undelegated; then a copied attack's source card run
/// for the copycat.
fn defender_side(g: &mut Game, acc: &mut crate::spec::passive::DamageAcc) -> R {
    use crate::spec::passive::Modifier;
    let order = g.decl_all(crate::decl::fam::DAMAGE_TAKEN, 0);
    if !order.is_empty() {
        crate::derived::Facts::undelegated(g, |g| -> R {
            for c in order.iter().copied() {
                for ps in crate::decl::passives(g, c) {
                    if let Modifier::DamageTaken(d) = &ps.modifier {
                        crate::spec::passive::damage_taken_into(g, c, ps.origin, d, acc)?;
                    }
                }
            }
            // The survive-on-10 replacements ("would be Knocked Out ... is not Knocked Out and its remaining HP becomes
            // 10") are one group, against the damage the step's other effects left (batch 8 C12c): two or more that
            // apply are ordered by the Pokémon's owner and a later one doesn't apply once the Pokémon is saved (JP FAQ
            // Annihilape + Survival Brace, Crustle + Focus Sash; id1014): the group waits until the attack's damage is
            // all done (`resolve_survive_groups`); one alone applies as it always did.
            let mut cands: SVec<SurviveCand, 4> = SVec::new();
            for c in order.iter().copied() {
                for ps in crate::decl::passives(g, c) {
                    if let Modifier::SurviveOnTen(s) = &ps.modifier {
                        if crate::spec::passive::survive_applies(g, c, ps.origin, s, acc)? && cands.len() < cands.capacity() {
                            cands.push(SurviveCand { card: c, origin: ps.origin, kind: s.kind });
                        }
                    }
                }
            }
            if cands.len() >= 2 {
                g.survive_groups.push(SurviveGroup { target: acc.b.target, cands });
            } else if let Some(sc) = cands.as_slice().first().copied() {
                let spec = crate::spec::passive::SurviveOnTenSpec { kind: sc.kind };
                crate::spec::passive::survive_on_ten_into(g, sc.card, sc.origin, &spec, acc)?;
            }
            Ok(())
        })?;
    }
    crate::copy_attack::delegate_damage(g, acc, true)
}

/// A survive-on-10 replacement that applies to a hit.
#[derive(Clone, Copy, Debug)]
pub struct SurviveCand {
    pub card: crate::list::CardId,
    pub origin: crate::spec::passive::RuleSource,
    pub kind: crate::spec::passive::SurviveKind,
}

/// Two or more survive-on-10 replacements of one hit, waiting for the owner's order.
#[derive(Clone, Copy, Debug)]
pub struct SurviveGroup {
    pub target: SlotRef,
    pub cands: SVec<SurviveCand, 4>,
}

/// The survive-on-10 groups of the attack (`defender_side`), once its damage is all done: the owner chooses which applies
/// first (a Select prompt over the cards), then the next, until one saves the Pokémon (a Tool is discarded, a coin is
/// flipped, only when it applies). Returns with a prompt open when a choice is asked (the attack resumes after it).
pub fn resolve_survive_groups(g: &mut Game) -> R {
    while let Some(grp) = g.survive_groups.pop() {
        survive_group_next(g, grp)?;
        if g.has_prompts() {
            return Ok(());
        }
    }
    Ok(())
}

fn survive_group_next(g: &mut Game, mut grp: SurviveGroup) -> R {
    let t = grp.target;
    let (tp, ts) = (t.p as usize, t.s);
    if g.st.slot_pokemon(tp, ts).is_none() || grp.cands.is_empty() {
        return Ok(());
    }
    let hp = crate::derived::hp(g, tp, ts)?;
    if g.st.slot(tp, ts).damage < hp {
        return Ok(());
    }
    if grp.cands.len() >= 2 {
        let mut names: SVec<(&'static str, Option<crate::list::CardId>), 8> = SVec::new();
        for c in grp.cands.iter() {
            names.push((g.st.cdef(c.card).full_name, Some(c.card)));
        }
        g.prompt(tp, crate::purpose::PromptPurpose::OrderEffects, crate::prompts::PromptKind::Select { values: crate::prompts::SelectValues::Effects(names), allow_cancel: false, default_value: 0 }, crate::prompts::Provenance::NONE, crate::game::Cont::SurviveOrder { grp });
        return Ok(());
    }
    let c = grp.cands.remove_at(0);
    survive_apply(g, t, c, hp)
}

/// The owner's choice: that replacement applies now; the next ones only if the Pokémon still would be Knocked Out.
pub fn survive_order_chosen(g: &mut Game, mut grp: SurviveGroup, choice: i32) -> R {
    let k = (choice.max(0) as usize).min(grp.cands.len().saturating_sub(1));
    if grp.cands.is_empty() {
        return resolve_survive_groups(g);
    }
    let c = grp.cands.remove_at(k);
    let t = grp.target;
    if g.st.slot_pokemon(t.p as usize, t.s).is_some() {
        let hp = crate::derived::hp(g, t.p as usize, t.s)?;
        if g.st.slot(t.p as usize, t.s).damage >= hp {
            survive_apply(g, t, c, hp)?;
        }
    }
    if g.has_prompts() {
        return Ok(());
    }
    survive_group_next(g, grp)?;
    if g.has_prompts() {
        return Ok(());
    }
    resolve_survive_groups(g)
}

/// One replacement of a group applies: a Tool's is discarded, a coin flipped (heads saves), a full-HP Ability saves.
fn survive_apply(g: &mut Game, t: SlotRef, c: SurviveCand, hp: i32) -> R {
    use crate::spec::passive::SurviveKind;
    let owner = t.p as usize;
    match c.kind {
        SurviveKind::IfFullHp => survive_on_10(g, t, hp),
        SurviveKind::ToolIfFullHp => {
            survive_on_10(g, t, hp);
            let cause = crate::cause::Cause::of_origin(c.origin, c.card, owner as u8);
            crate::engine::knockout::leave_play_cards(g, t, &[c.card], crate::spec::event::RulesZone::Discard, cause, None)?;
        }
        SurviveKind::OnCoin => {
            let cause = crate::cause::Cause::of_origin(c.origin, c.card, owner as u8);
            if crate::engine::condition::flip_coin(g, owner, crate::game::CoinCb::None, cause)? {
                survive_on_10(g, t, hp);
            }
        }
    }
    Ok(())
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
/// Energy, Battle Cage, ...; the record shows counters of 0 then, `record::comes_out_zero`).
pub fn place(g: &mut Game, target: SlotRef, hp: i32, cause: Cause) -> R<bool> {
    if hp <= 0 || g.st.slot_pokemon(target.p as usize, target.s).is_none() {
        return Ok(false);
    }
    let v = counters_view(g, target, hp, cause);
    if crate::engine::condition::refused(g, &v)?.is_some() {
        crate::record::comes_out_zero(g, EventKind::PlaceCounters, target, cause);
        return Ok(false);
    }
    g.run_fx_unit(Effect::PlaceCounters { p: target.p, target, amount: hp, cause })?;
    Ok(true)
}

/// MoveCounters: one action moving counters, `pairs` of (from, to, HP) in the order the text or the player gives them.
/// Each pair moves as much as it can (the counters its source has then, id63). Returns what happened to each pair
/// (removed from the source, placed on the destination); a pair moves nothing when a lock forbids moving counters from
/// its source or onto its destination (Patrat's Watchful Eye binds both players; asked per pair, batch 8 C12b: a lock can
/// spare a Pokémon, Hide 'n' Sneak against an opponent's Patrat).
pub fn move_counters(g: &mut Game, pairs: &[(SlotRef, SlotRef, i32)], cause: Cause) -> R<SVec<CounterMove, { crate::prompts::MAX_DAMAGE_RUNS }>> {
    let mut moves: SVec<CounterMove, { crate::prompts::MAX_DAMAGE_RUNS }> = SVec::new();
    if pairs.is_empty() {
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
        let lf = move_view(g, from, MoveEnd::From, removed, cause);
        let lt = move_view(g, to, MoveEnd::To, removed, cause);
        if crate::derived::event_locked(g, &lf)?.is_some() || crate::derived::event_locked(g, &lt)?.is_some() {
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
        moves.push(CounterMove { from, to, removed: removed as i16, placed: placed as i16 });
    }
    if !moves.is_empty() {
        g.fxp.moves.push(moves);
        let buf = (g.fxp.moves.len() - 1) as u8;
        let r = g.run_fx_unit(Effect::MoveCounters { p: cause.player, buf, cause });
        g.fxp.moves.pop();
        r?;
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
            *g.st.damage_mut(target.p as usize, target.s) += amount.max(0);
        }
        Effect::Damage { b, amount, survive } => {
            let t = b.target;
            let (tp, ts) = (t.p as usize, t.s);
            let Some(card) = g.st.slot_pokemon(tp, ts) else { crate::bail!("ILLEGAL_ACTION") };
            *g.st.damage_mut(tp, ts) += amount;
            g.st.cards[card as usize].damage_taken_last_turn += amount;
            // The survive-on-10 replacement a full-HP Pokémon's effect armed in the calculation (its remaining HP is 10).
            if survive {
                // The damage was just placed: the HP is read after it (a write).
                g.st.bump();
                let hp = crate::derived::hp(g, tp, ts)?;
                if g.st.slot(tp, ts).damage >= hp {
                    survive_on_10(g, t, hp);
                }
            }
            // What the attack damaged, for "Knocked Out by damage from an attack" (the opponent's Pokémon, any spot; and
            // those it damaged in the Active Spot). Damage is not an effect: written whatever protects the Pokémon.
            if t.p != b.player && g.st.phase == crate::types::GamePhase::Attack {
                if let Some(la) = g.last_attack.as_mut() {
                    if !la.damaged.contains(&t) {
                        la.damaged.push(t);
                    }
                    if g.st.players[tp].active == ts {
                        la.damage_to_active += amount;
                        if !la.damaged_active.contains(&t) {
                            la.damaged_active.push(t);
                        }
                    }
                }
            }
        }
        Effect::MoveCounters { buf, .. } => {
            let moves = g.fxp.moves[buf as usize];
            for m in moves.iter() {
                let s = g.st.damage_mut(m.from.p as usize, m.from.s);
                *s = (*s - m.removed as i32).max(0);
                *g.st.damage_mut(m.to.p as usize, m.to.s) += m.placed as i32;
            }
        }
        _ => {}
    }
    Ok(())
}

/// A delayed trap on the damaged Pokémon ("if this Pokémon is damaged by an attack during your opponent's next turn, put N
/// damage counters on the Attacking Pokémon"; `engine::lasting`), recorded in the attack's E-03 window when the damage is done.
#[derive(Clone, Copy, Debug)]
pub struct Retaliation {
    /// The damaged Pokémon.
    pub target: SlotRef,
    /// The Attacking Pokémon's slot and the Pokémon card that was in it.
    pub source: SlotRef,
    pub source_pokemon: Option<crate::list::CardId>,
    /// The lasting trigger (`LastingWhat::Retaliate`).
    pub lasting: crate::state::LastingEffect,
    /// The damage the attack did to the damaged Pokémon (what "equal to the damage done" reads).
    pub damage: i32,
}

/// The HP of damage counters the trap puts on the Attacking Pokémon.
fn retaliation_hp(t: &Retaliation) -> i32 {
    match t.lasting.what {
        crate::state::LastingWhat::Retaliate { amount: crate::spec::ops::state::PutOnAttacker::Counters(n), .. } => n as i32,
        crate::state::LastingWhat::Retaliate { amount: crate::spec::ops::state::PutOnAttacker::DamageDone, .. } => t.damage,
        _ => 0,
    }
}

/// The Damage event `id` is done: the damaged Pokémon's lasting trigger records itself with the attack's other
/// "is damaged" effects (the cards' triggers recorded first, `run::fire_on`; APR E-03), or applies at once when no window is
/// open.
pub fn record_retaliation(g: &mut Game, id: EffId) -> R {
    let Effect::Damage { b, amount: dealt, .. } = *g.e(id) else { return Ok(()) };
    let t = b.target;
    if t.p == b.player || g.st.phase != crate::types::GamePhase::Attack {
        return Ok(());
    }
    let Some(r) = crate::engine::lasting::armed_retaliate(&g.st, t.p as usize, t.s) else { return Ok(()) };
    let rec = Retaliation { target: t, source: b.source, source_pokemon: g.st.slot_pokemon(b.source.p as usize, b.source.s), lasting: r, damage: dealt };
    if retaliation_hp(&rec) <= 0 {
        return Ok(());
    }
    if g.triggers_open(b.attack_effect) {
        crate::engine::order::record(g, crate::engine::order::Tag::Damaged(b.attack_effect), crate::engine::order::Item::Retaliation(rec));
        return Ok(());
    }
    retaliate(g, &rec)
}

/// The Attacking Pokémon is still in play (ruling 530) and the trap's source is still on the damaged Pokémon (an effect of
/// the damaged Pokémon, gone when it left play).
fn retaliation_holds(g: &Game, t: &Retaliation) -> bool {
    let in_play = t.source_pokemon.is_some() && g.st.slot_pokemon(t.source.p as usize, t.source.s) == t.source_pokemon;
    in_play && g.st.slot(t.target.p as usize, t.target.s).cards().contains(t.lasting.source)
}

/// Does the trap apply now: it puts counters (APR E-03: the effects that apply are ordered)?
pub fn retaliation_applies(g: &Game, t: &Retaliation) -> bool {
    retaliation_hp(t) > 0 && retaliation_holds(g, t)
}

/// Resolve the trap: the counters go on the Attacking Pokémon wherever it is (rulings 482, 1839), an effect of the damaged
/// Pokémon's own earlier attack (id2408, id1958: a PlaceCounters by that attack, so Mist Energy stops it).
pub fn retaliate(g: &mut Game, t: &Retaliation) -> R {
    let crate::state::LastingWhat::Retaliate { attack, .. } = t.lasting.what else { return Ok(()) };
    if retaliation_applies(g, t) {
        let cause = Cause::attack(t.target.p, Some(t.lasting.source), attack);
        place(g, t.source, retaliation_hp(t), cause)?;
    }
    Ok(())
}

/// "It is not Knocked Out, and its remaining HP becomes 10" (ruling 1770; survive-on-10 replacements): the damage that
/// leaves 10 HP of `hp`, and the Pokémon is recorded so an effect of the same attack lowering its HP keeps it at 10
/// (ruling 1589; `engine::phase`'s AfterAttack).
pub fn survive_on_10(g: &mut Game, t: SlotRef, hp: i32) {
    *g.st.damage_mut(t.p as usize, t.s) = hp - 10;
    if !g.ten_hp.contains(&t) {
        g.ten_hp.push(t);
    }
}

/// The remaining HP of a Pokémon recorded by [`survive_on_10`] is 10 again after the attack lowered its HP (ruling 1589).
pub fn keep_10(g: &mut Game, t: SlotRef, hp: i32) {
    *g.st.damage_mut(t.p as usize, t.s) = hp - 10;
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
    const ZAPDOS: &str = "Zapdos TWM 65";

    fn game(sc: serde_json::Value) -> Game {
        let mut names: Vec<&str> = Vec::new();
        for n in [MILOTIC, OGERPON, SNORLAX, MIST, ZAPDOS] {
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
        let atk = g.new_fx(Effect::Attack { p: p as u8, opp: (1 - p) as u8, attack, damage: 0, ignore_weakness: true, ignore_resistance: true, ignore_defender_effects: false, source, second_attack: false });
        g.st.set_phase(GamePhase::Attack);
        g.last_attack = Some(crate::game::LastAttack { p: p as u8, effect: atk, attack, source, pokemon: Some(card), damaged_active: SVec::new(), damage_to_active: 0, damaged: SVec::new() });
        AtkBase { attack_effect: atk, player: p as u8, opponent: (1 - p) as u8, attack, source, target, cause: Cause::of_attack_at(g, p as u8, attack, source) }
    }

    /// Weakness and Resistance apply to the attack's damage to the attacker itself (APR B-09: "Weakness, Resistance, and
    /// other effects on the Pokémon still apply"), never to a Benched Pokémon (APR B-08), and not on the `Put` route.
    /// Zapdos TWM 65 is a Lightning Pokémon with Lightning Weakness (no self-damage user in the pool is weak or resistant
    /// to its own type: latent).
    #[test]
    fn weakness_applies_to_self_damage() {
        let mut g = game(json!({"me": {"reset": true, "active": ZAPDOS, "bench": [{"card": ZAPDOS}]}, "opp": {"reset": true, "active": SNORLAX}}));
        let me = g.st.active_player as usize;
        let own = SlotRef::new(me, g.st.players[me].active);
        let bench = SlotRef::new(me, g.st.players[me].bench.as_slice()[0]);
        let mut b = attack_on(&mut g, me, own);
        if let Effect::Attack { ignore_weakness, ignore_resistance, .. } = g.e_mut(b.attack_effect) {
            *ignore_weakness = false;
            *ignore_resistance = false;
        }
        deal(&mut g, b, 30, true).unwrap();
        assert_eq!(g.st.slot(me, own.s).damage, 60, "the attacker's damage to itself: Weakness x2 (APR B-09)");
        b.target = bench;
        deal(&mut g, b, 30, true).unwrap();
        assert_eq!(g.st.slot(me, bench.s).damage, 30, "no Weakness on a Benched Pokémon (APR B-08)");
        b.target = own;
        deal(&mut g, b, 10, false).unwrap();
        assert_eq!(g.st.slot(me, own.s).damage, 70, "the Put route: no Weakness");
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
        g.st.lasting_mut().push(crate::state::LastingEffect {
            on: crate::state::LastingOn::pokemon(o, t.s),
            source: 0,
            what: crate::state::LastingWhat::TakesMoreDamage { amount: 30, attacker: Some(me as u8), pending: false, rearm: false },
        });
        let b = attack_on(&mut g, me, t);
        deal(&mut g, b, 100, true).unwrap();
        assert_eq!(g.st.slot(o, t.s).damage, 0, "Sparkling Scales prevents the Tera Pokémon's damage, the 30 more included");
        // A non-Tera attacker's damage (Milotic ex's own) gets the 30 more.
        let mut g = game(json!({"me": {"reset": true, "active": MILOTIC}, "opp": {"reset": true, "active": MILOTIC}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let t = SlotRef::new(o, g.st.players[o].active);
        g.st.lasting_mut().push(crate::state::LastingEffect {
            on: crate::state::LastingOn::pokemon(o, t.s),
            source: 0,
            what: crate::state::LastingWhat::TakesMoreDamage { amount: 30, attacker: Some(me as u8), pending: false, rearm: false },
        });
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

    /// One action is one MoveCounters event however many pairs it has (id66): up to `prompts::MAX_DAMAGE_RUNS`, the
    /// most runs a MOVE_DAMAGE answer holds.
    #[test]
    fn one_event_carries_every_pair() {
        let mut g = game(json!({"me": {"reset": true, "active": SNORLAX}, "opp": {"reset": true, "active": SNORLAX, "active_damage": 100, "bench": [{"card": SNORLAX}]}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let act = SlotRef::new(o, g.st.players[o].active);
        let b = SlotRef::new(o, g.st.players[o].bench.as_slice()[0]);
        let cause = attack_on(&mut g, me, act).cause;
        let pairs: Vec<(SlotRef, SlotRef, i32)> = (0..crate::prompts::MAX_DAMAGE_RUNS).map(|i| if i % 2 == 0 { (act, b, 10) } else { (b, act, 10) }).collect();
        let m = move_counters(&mut g, &pairs, cause).unwrap();
        assert_eq!(m.len(), crate::prompts::MAX_DAMAGE_RUNS);
        assert_eq!((g.st.slot(o, act.s).damage, g.st.slot(o, b.s).damage), (100, 0));
    }

    /// A counter-move answer moves each counter from a Pokémon that has one at that point (counters it received earlier
    /// in the answer included): a move from a Pokémon without any would do nothing and isn't a valid answer (Munkidori,
    /// Alakazam TWM; legality = declared validity).
    #[test]
    fn a_counter_moves_only_from_a_pokemon_that_has_one() {
        use crate::prompts::*;
        use crate::types::{PlayerType, SlotType};
        let mut g = game(json!({"me": {"reset": true, "active": SNORLAX}, "opp": {"reset": true, "active": SNORLAX, "active_damage": 20, "bench": [{"card": SNORLAX}, {"card": SNORLAX}]}}));
        let me = g.st.active_player as usize;
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let stashed_o = g.stash_opts(MoveOpts::default());
        g.prompt(me, crate::purpose::PromptPurpose::MoveDamage, PromptKind::MoveDamage { player_type: PlayerType::TopPlayer, slots, max_allowed: SVec::new(), o: stashed_o, single_source: false, single_destination: false, damage_multiple: 10 }, crate::prompts::Provenance::NONE, crate::game::Cont::Noop);
        let pr = *g.prompts.as_slice().last().unwrap();
        let act = json!({"player": 1, "slot": 1, "index": 0});
        let b0 = json!({"player": 1, "slot": 2, "index": 0});
        let b1 = json!({"player": 1, "slot": 2, "index": 1});
        let mv = |f: &serde_json::Value, t: &serde_json::Value| json!({"from": f, "to": t});
        assert!(g.decode_answer(&pr, &json!([mv(&act, &b0), mv(&act, &b0)])).is_ok());
        assert!(g.decode_answer(&pr, &json!([mv(&act, &b0), mv(&b0, &b1)])).is_ok(), "counters received earlier can move on");
        assert!(g.decode_answer(&pr, &json!([mv(&b1, &b0)])).is_err(), "no counter on the source: nothing to move");
        assert!(g.decode_answer(&pr, &json!([mv(&act, &b0), mv(&act, &b0), mv(&act, &b0)])).is_err(), "only 2 counters there");
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
