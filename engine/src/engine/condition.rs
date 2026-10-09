//! The events of events batch 4 (docs/design/events-design.md, section 4): GainCondition, RemoveCondition,
//! RemoveCounters (healing) and CoinFlip.
//!
//! Every path that gives a Pokémon a Special Condition, makes it recover from one, heals it, or flips a coin calls
//! its routine here, in the `engine::enter` / `engine::attach` pattern: the routine checks the event (the Pokémon
//! is there; the locks, `derived::event_locked`; the preventions, `derived::event_prevented`), produces it (an `Effect` dispatched to the cards), and applies
//! its consequences in [`reducer`]. A refused event doesn't happen (`Ok(false)`): an effect does as much as it can.
//!
//! - [`gain`]: "is now Poisoned / Burned / Asleep / Paralyzed / Confused", whatever causes it (an attack, an
//!   Ability, a Trainer, a Tool, Janine's Secret Art). One event per condition. The attack's and the Ability's
//!   effects first go through their B4-OLD probes (`Effect::AddSpecialConditions`,
//!   `AddSpecialConditionsPower`), which the attack-effect preventions and Hide 'n' Sneak still read; their
//!   reducers call [`gain`].
//! - [`remove`] / [`recover_all`]: the Pokémon recovers from a condition: an effect ("recovers from all Special
//!   Conditions"), evolving or devolving, moving to the Bench (retreat, switch), the Checkup (Paralyzed at the end
//!   of its owner's turn, a heads for Burned or Asleep). One event per condition.
//! - [`heal`]: damage counters removed from a Pokémon (APR C-06). Moving counters off a Pokémon isn't healing it
//!   (events design 4.1; batch 6).
//! - [`coin_flipped`]: one event per physical flip, once its result is known, whatever asked for it: a card's
//!   "flip a coin" (the B4-OLD `CoinFlipRequest` / `CoinFlipSequence`, which Backtrack Badge's re-flip hook
//!   replaces), the Checkup flips for Burned and Asleep, the flip of a Confused Pokémon trying to attack, the flip
//!   for who goes first.
//!
//! The Cause is the frame's (`spec::run::Frame::cause`) or the rule's; ops never pick one.

use crate::cause::{Cause, CauseKind, RuleWhich};
use crate::effects::{EffId, Effect, SlotRef};
use crate::game::{Game, R};
use crate::spec::event::*;
use crate::state::Slot;
use crate::types::SpecialCondition;

// ---------------------------------------------------------------------------
// Event views

/// The GainCondition / RemoveCondition event of `condition` on the Pokémon in `target`. The event's card is that
/// Pokémon, its owner the Pokémon's owner.
pub fn condition_view(g: &Game, kind: EventKind, target: SlotRef, condition: SpecialCondition, cause: Cause) -> EventView {
    EventView { card: g.st.slot_pokemon(target.p as usize, target.s), slot: Some(target), condition: Some(condition), ..EventView::new(kind, cause, target.p, g.st.active_player) }
}

/// The RemoveCounters event: `amount` HP of damage counters off the Pokémon in `target`.
pub fn heal_view(g: &Game, target: SlotRef, amount: i32, cause: Cause) -> EventView {
    EventView { card: g.st.slot_pokemon(target.p as usize, target.s), slot: Some(target), amount, ..EventView::new(EventKind::RemoveCounters, cause, target.p, g.st.active_player) }
}

/// The CoinFlip event of player `p`.
pub fn coin_view(g: &Game, p: usize, purpose: CoinPurpose, heads: bool, cause: Cause) -> EventView {
    EventView { purpose: Some(purpose), heads: Some(heads), ..EventView::new(EventKind::CoinFlip, cause, p as u8, g.st.active_player) }
}

// ---------------------------------------------------------------------------
// The checks

/// The refusal of a prevented event (`derived::event_prevented`).
pub const PREVENTED: &str = "PREVENTED";

/// The checks every event of this module makes after its own (a Pokémon there, something to remove): the locks
/// (`derived::event_locked`), then the preventions (`derived::event_prevented`: "this Pokémon can't be Confused",
/// whatever the cause). `Some(code)` is a refusal (the event doesn't happen); an error of a read is propagated.
pub fn refused(g: &mut Game, v: &EventView) -> R<Option<&'static str>> {
    if let Some(code) = crate::derived::event_locked(g, v)? {
        return Ok(Some(code));
    }
    if crate::derived::event_prevented(g, v)? {
        return Ok(Some(PREVENTED));
    }
    Ok(None)
}

/// The damage counters a condition's rule places, before any effect adds more: Poison 1 at each Checkup, Burn 2,
/// Confusion 3 when the attack fails; none for Asleep and Paralyzed.
pub const fn base_counters(c: SpecialCondition) -> u8 {
    match c {
        SpecialCondition::Poisoned => 1,
        SpecialCondition::Burned => 2,
        SpecialCondition::Confused => 3,
        SpecialCondition::Asleep | SpecialCondition::Paralyzed => 0,
    }
}

/// The cause of what a Special Condition's own rule does (its Checkup flip, a Confused Pokémon's flip):
/// `CauseKind::SpecialCondition`, for the Pokémon's owner.
pub const fn by_condition(p: usize) -> Cause {
    Cause::new(CauseKind::SpecialCondition, None, p as u8)
}

/// Pokémon Checkup, for the owner `p` of the Pokémon (recovering at the Checkup).
pub const fn by_checkup(p: usize) -> Cause {
    Cause::rule(RuleWhich::Checkup, p as u8)
}

// ---------------------------------------------------------------------------
// The routines

/// GainCondition: the Pokémon in `target` is now affected by `condition`. It doesn't happen (`Ok(false)`) when no
/// Pokémon is there or the event is refused.
pub fn gain(g: &mut Game, target: SlotRef, condition: SpecialCondition, cause: Cause) -> R<bool> {
    if g.st.slot_pokemon(target.p as usize, target.s).is_none() {
        return Ok(false);
    }
    let v = condition_view(g, EventKind::GainCondition, target, condition, cause);
    if refused(g, &v)?.is_some() {
        return Ok(false);
    }
    g.run_fx_unit(Effect::GainCondition { p: target.p, target, condition, counters: base_counters(condition), cause })?;
    Ok(true)
}

/// RemoveCondition: the Pokémon in `target` recovers from `condition`. It doesn't happen when it isn't affected by
/// it or the event is refused.
pub fn remove(g: &mut Game, target: SlotRef, condition: SpecialCondition, cause: Cause) -> R<bool> {
    if !g.st.slot(target.p as usize, target.s).special_conditions.contains(&(condition as u8)) {
        return Ok(false);
    }
    let v = condition_view(g, EventKind::RemoveCondition, target, condition, cause);
    if refused(g, &v)?.is_some() {
        return Ok(false);
    }
    g.run_fx_unit(Effect::RemoveCondition { p: target.p, target, condition, cause })?;
    Ok(true)
}

/// The Pokémon in `target` recovers from every Special Condition it has except those in `keep` (the ones a card
/// preserves when it evolves: `CheckSpecialConditionRemoval`): one RemoveCondition each, in the order it got them.
pub fn recover_all(g: &mut Game, target: SlotRef, cause: Cause, keep: &[u8]) -> R {
    let conds = g.st.slot(target.p as usize, target.s).special_conditions;
    for c in conds.iter().copied() {
        if !keep.contains(&c) {
            remove(g, target, SpecialCondition::from_u8(c), cause)?;
        }
    }
    Ok(())
}

/// RemoveCounters (healing): `amount` HP of damage counters come off the Pokémon in `target` (at most its damage:
/// the event reports what is removed, not what the text asked for). It doesn't happen
/// when there is nothing to heal (no Pokémon, no damage, no amount), when a lasting effect says the Pokémon can't
/// be healed (`Slot::cannot_be_healed_next_turn`), or when the event is refused.
pub fn heal(g: &mut Game, target: SlotRef, amount: i32, cause: Cause) -> R<bool> {
    if g.st.slot_pokemon(target.p as usize, target.s).is_none() {
        return Ok(false);
    }
    let slot = g.st.slot(target.p as usize, target.s);
    if amount <= 0 || slot.damage <= 0 || slot.cannot_be_healed_next_turn {
        return Ok(false);
    }
    // The event is the counters actually removed: "heal 60" on a Pokémon with 30 damage removes 30.
    let amount = amount.min(slot.damage);
    let v = heal_view(g, target, amount, cause);
    if refused(g, &v)?.is_some() {
        return Ok(false);
    }
    g.run_fx_unit(Effect::Heal { p: target.p, target, damage: amount, cause })?;
    Ok(true)
}

/// CoinFlip: player `p` flipped a coin for `purpose` and got `heads` (or tails). The flip has happened (its result
/// stands for whatever asked for it): a refusal only means nothing reacts to it. No rules text forbids or prevents
/// flipping a coin; a declaration that does would need a ruling on what the flip's result is.
pub fn coin_flipped(g: &mut Game, p: usize, purpose: CoinPurpose, heads: bool, cause: Cause) -> R {
    let v = coin_view(g, p, purpose, heads, cause);
    if refused(g, &v)?.is_some() {
        return Ok(());
    }
    g.run_fx_unit(Effect::CoinFlip { p: p as u8, purpose, heads, cause })
}

// ---------------------------------------------------------------------------
// Consequences, applied by the events' reducer

/// The reducer of this module's events: what each does to the game.
pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::GainCondition { target, condition, counters, .. } => {
            put_condition(&mut g.st.players[target.p as usize].slots[target.s as usize], condition, counters);
            Ok(())
        }
        Effect::RemoveCondition { target, condition, .. } => {
            let v = condition as u8;
            g.st.players[target.p as usize].slots[target.s as usize].special_conditions.retain(|x| *x != v);
            Ok(())
        }
        Effect::Heal { target, damage, .. } => {
            let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
            if damage > 0 && slot.damage > 0 {
                slot.healed_this_turn = true;
            }
            slot.damage = (slot.damage - damage).max(0);
            Ok(())
        }
        _ => Ok(()),
    }
}

/// The slot's Pokémon is now affected by `sc`, whose rule places `counters` damage counters (Poison and Burn at
/// each Checkup, Confusion when the attack fails). Asleep, Confused and Paralyzed replace one another (the newest
/// stays); Poisoned and Burned stack with them. Gaining a condition the Pokémon has sets its counters again.
pub fn put_condition(slot: &mut Slot, sc: SpecialCondition, counters: u8) {
    let hp = counters as i32 * 10;
    match sc {
        SpecialCondition::Poisoned => slot.poison_damage = hp,
        SpecialCondition::Burned => slot.burn_damage = hp,
        SpecialCondition::Confused => slot.confusion_damage = hp,
        _ => {}
    }
    let v = sc as u8;
    if slot.special_conditions.contains(&v) {
        return;
    }
    if sc == SpecialCondition::Poisoned || sc == SpecialCondition::Burned {
        slot.special_conditions.push(v);
        return;
    }
    slot.special_conditions.retain(|s| !(*s == SpecialCondition::Paralyzed as u8 || *s == SpecialCondition::Confused as u8 || *s == SpecialCondition::Asleep as u8));
    slot.special_conditions.push(v);
}

#[cfg(test)]
mod tests {
    //! The routines against hand-built boards.
    use super::*;
    use crate::cause::CauseKind;
    use serde_json::json;

    const SLOWPOKE: &str = "Slowpoke MEP 86";
    const SNORLAX: &str = "Hop's Snorlax JTG 117";
    const YVELTAL: &str = "Yveltal 30C 100";

    fn game(sc: serde_json::Value) -> Game {
        let mut names: Vec<&str> = Vec::new();
        for n in [SLOWPOKE, SNORLAX, YVELTAL] {
            names.extend(std::iter::repeat(n).take(4));
        }
        while names.len() < 60 {
            names.push("Psychic Energy MEE 5");
        }
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    fn active(g: &Game, p: usize) -> SlotRef {
        SlotRef::new(p, g.st.players[p].active)
    }

    fn trainer(p: usize) -> Cause {
        Cause::new(CauseKind::Trainer, None, p as u8)
    }

    /// "This Pokémon can't be Confused" stops a Confusion whatever causes it, and nothing else.
    #[test]
    fn dopey_face_prevents_confusion_by_any_cause() {
        let mut g = game(json!({"me": {"reset": true, "active": SNORLAX}, "opp": {"reset": true, "active": SLOWPOKE}}));
        let me = g.st.active_player as usize;
        let t = active(&g, 1 - me);
        assert!(!gain(&mut g, t, SpecialCondition::Confused, trainer(me)).unwrap(), "prevented");
        assert!(g.st.slot(t.p as usize, t.s).special_conditions.is_empty());
        assert!(gain(&mut g, t, SpecialCondition::Burned, trainer(me)).unwrap());
        let a = Cause::new(CauseKind::Ability, None, me as u8);
        assert!(!gain(&mut g, t, SpecialCondition::Confused, a).unwrap(), "an Ability too");
        // Not another Pokémon.
        let mine = active(&g, me);
        assert!(gain(&mut g, mine, SpecialCondition::Confused, trainer(me)).unwrap());
        assert_eq!(g.st.slot(me, mine.s).confusion_damage, 30);
    }

    /// Asleep, Confused and Paralyzed replace one another; Poisoned and Burned stack; recovering removes one.
    #[test]
    fn conditions_replace_and_recover() {
        let mut g = game(json!({"me": {"reset": true, "active": SNORLAX}, "opp": {"reset": true, "active": SNORLAX}}));
        let me = g.st.active_player as usize;
        let t = active(&g, 1 - me);
        for c in [SpecialCondition::Poisoned, SpecialCondition::Asleep, SpecialCondition::Paralyzed] {
            assert!(gain(&mut g, t, c, trainer(me)).unwrap());
        }
        let conds = |g: &Game| g.st.slot(t.p as usize, t.s).special_conditions.as_slice().to_vec();
        assert_eq!(conds(&g), vec![SpecialCondition::Poisoned as u8, SpecialCondition::Paralyzed as u8]);
        assert!(!remove(&mut g, t, SpecialCondition::Asleep, trainer(me)).unwrap(), "nothing to remove");
        recover_all(&mut g, t, trainer(me), &[SpecialCondition::Poisoned as u8]).unwrap();
        assert_eq!(conds(&g), vec![SpecialCondition::Poisoned as u8]);
    }

    /// Healing: nothing happens without damage counters; "your opponent's Active Pokémon can't be healed"
    /// (Yveltal's Life-Locked) is a prevention of the RemoveCounters event.
    #[test]
    fn heal_and_life_locked() {
        let mut g = game(json!({"me": {"reset": true, "active": YVELTAL}, "opp": {"reset": true, "active": SNORLAX, "active_damage": 50, "bench": [{"card": SNORLAX, "damage": 30}]}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let t = active(&g, o);
        let b = SlotRef::new(o, g.st.players[o].bench.as_slice()[0]);
        assert!(!heal(&mut g, t, 30, trainer(o)).unwrap(), "Life-Locked");
        assert_eq!(g.st.slot(o, t.s).damage, 50);
        assert!(heal(&mut g, b, 50, trainer(o)).unwrap(), "a Benched Pokémon can be healed");
        assert_eq!(g.st.slot(o, b.s).damage, 0);
        assert!(g.st.slot(o, b.s).healed_this_turn);
        assert!(!heal(&mut g, b, 10, trainer(o)).unwrap(), "no damage counters left");
        // Yveltal's own side heals.
        g.st.players[me].slots[g.st.players[me].active as usize].damage = 20;
        let mine = active(&g, me);
        assert!(heal(&mut g, mine, 10, trainer(me)).unwrap());
    }
}
