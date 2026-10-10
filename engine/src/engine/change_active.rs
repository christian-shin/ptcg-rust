//! The ChangeActive event of events batch 5 (docs/design/events-design.md, sections 4 and 4.4): the Active Pokémon
//! changes.
//!
//! Every path that changes a player's Active Pokémon calls [`change_active`] (or, for the retreat, [`check`] before
//! paying the cost and [`produce`] after it), in the `engine::enter` / `engine::attach` / `engine::condition` pattern:
//! the routine checks the event (the same function legality calls, [`check_with`]), produces it (an `Effect`
//! dispatched to the cards), applies its consequences in [`reducer`], and then the triggers over the event run
//! (`spec::run::after_event`; `Game::reduce_effect` re-stamps the Ability locks before them, since the Active Spot
//! changed).
//!
//! - Retreat (the turn action, `engine::retreat`; APR A-03).
//! - Switch (C-03), switch-in (C-05), switch-out (C-04): the switch ops of Trainers, Abilities and attacks
//!   (`spec::ops::board`: `Op::Switch`, `Op::SwitchWithActive`).
//! - Promotion: a Benched Pokémon goes to the empty Active Spot after a Knock Out (`engine::check`).
//!
//! The game's setup puts the Active Pokémon by EnterPlay, and the scenario loader arranges the board directly: neither
//! is a change of the Active Pokémon (being put into the Active Spot at setup isn't moving from the Bench).
//!
//! The change is done to one Pokémon (`ActiveChange::done_to_leaving`): the Active Pokémon for a retreat, a switch
//! and a switch-out, the Benched Pokémon brought in for a switch-in (APR C-04 / C-05 specific cases; id42, id2025,
//! id2155; official JP Q&A on Hariyama's Heave-Ho Catcher). The event's card and spot are that Pokémon, which is what
//! "prevent all effects of attacks / Abilities done to this Pokémon" protects (`Prevent` over ChangeActive). A
//! refused change doesn't happen (`Ok(false)`), and the text's "if you do" fails (`Cond::Done`).

use crate::cause::Cause;
use crate::effects::{EffId, Effect, SlotRef};
use crate::game::{Game, R};
use crate::list::*;
use crate::markers::*;
use crate::spec::event::*;
use crate::state::*;

/// One change of player `p`'s Active Pokémon: the Pokémon in the Active Spot `from` (the empty spot for a promotion)
/// goes to the Bench and the Benched Pokémon in `to` becomes the Active Pokémon. `to` is `None` only while it isn't
/// chosen yet (a switch-out is checked before the opponent chooses: id2025).
#[derive(Clone, Copy, Debug)]
pub struct ChangeActiveView {
    pub p: usize,
    pub from: SlotId,
    pub to: Option<SlotId>,
    pub change: ActiveChange,
    pub cause: Cause,
}

impl ChangeActiveView {
    /// The change of `p`'s Active Pokémon (the spot as it is now) with the Benched Pokémon in `to`.
    pub fn of(g: &Game, p: usize, to: Option<SlotId>, change: ActiveChange, cause: Cause) -> ChangeActiveView {
        ChangeActiveView { p, from: g.st.players[p].active, to, change, cause }
    }
}

// ---------------------------------------------------------------------------
// Event view

/// The ChangeActive event as predicates read it: `from` / `to` the two spots, `card` / `slot` the Pokémon the change
/// is done to, the owner the side's player.
pub fn view(g: &Game, c: &ChangeActiveView) -> EventView {
    let from = SlotRef::new(c.p, c.from);
    let to = c.to.map(|s| SlotRef::new(c.p, s));
    let slot = if c.change.done_to_leaving() { Some(from) } else { to };
    EventView {
        change: Some(c.change),
        from: Some(from),
        to,
        card: slot.and_then(|s| g.st.slot_pokemon(s.p as usize, s.s)),
        slot,
        ..EventView::new(EventKind::ChangeActive, c.cause, c.p as u8, crate::spec::event::whose_turn(g))
    }
}

/// The event view of a ChangeActive effect (after it: the two spots keep their Pokémon, now swapped).
pub fn effect_view(g: &Game, p: u8, from: SlotId, to: SlotId, change: ActiveChange, cause: Cause) -> EventView {
    view(g, &ChangeActiveView { p: p as usize, from, to: Some(to), change, cause })
}

// ---------------------------------------------------------------------------
// The checks (execution and legality call the same function)

/// The refusal of a prevented change (`derived::event_prevented`).
pub const PREVENTED: &str = "PREVENTED";

/// The spots allow the change (a plain read): `from` is the player's Active Spot, holding a Pokémon unless the change
/// is a promotion; `to` is one of the player's Benched Pokémon (or, not chosen yet, the player has one).
pub fn target_ok(g: &Game, v: &EventView) -> R {
    let (Some(change), Some(from)) = (v.change, v.from) else { crate::bail!("INVALID_TARGET") };
    let p = v.owner as usize;
    let pl = &g.st.players[p];
    if from.p as usize != p || from.s != pl.active {
        crate::bail!("INVALID_TARGET");
    }
    let leaving = g.st.slot_pokemon(p, from.s).is_some();
    if leaving == (change == ActiveChange::Promotion) {
        crate::bail!("INVALID_TARGET");
    }
    match v.to {
        Some(to) => {
            if to.p as usize != p || pl.bench_index_of(to.s).is_none() || g.st.slot_pokemon(p, to.s).is_none() {
                crate::bail!("INVALID_TARGET");
            }
        }
        None => {
            if !pl.bench.iter().any(|b| g.st.slot_pokemon(p, *b).is_some()) {
                crate::bail!("INVALID_TARGET");
            }
        }
    }
    Ok(())
}

/// The Pokémon's own lasting effects that forbid the change (a plain read): "this Pokémon can't retreat during your
/// opponent's next turn", an attack's effect stored on the spot (`Slot::cannot_retreat_next_turn`; a declaration with
/// the derived layer, events batch 8). It forbids retreating only: the Pokémon can still be switched (APR C-03).
pub fn lasting_refusal(g: &Game, v: &EventView) -> Option<&'static str> {
    let from = v.from?;
    (v.change == Some(ActiveChange::Retreat) && g.st.slot(from.p as usize, from.s).cannot_retreat_next_turn).then_some("BLOCKED_BY_EFFECT")
}

/// Where the checks of a ChangeActive read the game: execution on the game itself, legality (`legal.rs Ctx`) on its
/// scratch game behind its plain-read gates. Both answer through [`check_with`], so they can't drift.
pub trait ActiveChecks {
    /// The game the plain reads (the spots, the lasting effects) look at.
    fn game(&self) -> &Game;
    /// The one lock query (`derived::event_locked`): the code of the lock that forbids the event.
    fn event_locked(&mut self, v: &EventView) -> R<Option<&'static str>>;
    /// Is the event prevented (`derived::event_prevented`)? Legality, which checks the retreat (a turn action
    /// execution would refuse is illegal), reads the lasting effect only: no declaration matches a rule's cause
    /// (`no_prevention_matches_a_rule_cause`).
    fn event_prevented(&mut self, v: &EventView) -> R<bool>;
}

impl ActiveChecks for Game {
    fn game(&self) -> &Game {
        self
    }
    fn event_locked(&mut self, v: &EventView) -> R<Option<&'static str>> {
        crate::derived::event_locked(self, v)
    }
    fn event_prevented(&mut self, v: &EventView) -> R<bool> {
        crate::derived::event_prevented(self, v)
    }
}

/// Every check of a ChangeActive, in order: the spots ([`target_ok`]), the Pokémon's lasting effects
/// ([`lasting_refusal`]), the locks ("this Pokémon can't retreat": `Change(Retreat) & From(This)`), the preventions
/// ("prevent all effects of attacks done to this Pokémon": Mist Energy against an attack's switch-in of the Benched
/// Pokémon it is attached to). `Ok(Some(code))` when the event is refused (it doesn't happen); an error of a read is
/// propagated, never taken for a refusal.
pub fn check_with<C: ActiveChecks + ?Sized>(c: &mut C, v: &EventView) -> R<Option<&'static str>> {
    if let Err(e) = target_ok(c.game(), v) {
        return Ok(Some(e.0));
    }
    if let Some(code) = lasting_refusal(c.game(), v) {
        return Ok(Some(code));
    }
    if let Some(code) = c.event_locked(v)? {
        return Ok(Some(code));
    }
    if c.event_prevented(v)? {
        return Ok(Some(PREVENTED));
    }
    Ok(None)
}

/// [`check_with`] on the game, a refusal as its error (the retreat: a refused one is illegal).
pub fn check(g: &mut Game, c: &ChangeActiveView) -> R {
    let v = view(g, c);
    match check_with(g, &v)? {
        Some(code) => crate::bail!(code),
        None => Ok(()),
    }
}

/// Is the change refused (checked on the game)?
pub fn refused(g: &mut Game, c: &ChangeActiveView) -> R<bool> {
    let v = view(g, c);
    Ok(check_with(g, &v)?.is_some())
}

// ---------------------------------------------------------------------------
// The routines

/// ChangeActive by an effect or a rule: checked, then produced. It doesn't happen (`Ok(false)`) when it is refused
/// (no Benched Pokémon there, a lock, a prevention); "if you do" then fails.
pub fn change_active(g: &mut Game, c: ChangeActiveView) -> R<bool> {
    if c.to.is_none() || refused(g, &c)? {
        return Ok(false);
    }
    produce(g, c)?;
    Ok(true)
}

/// Produce the ChangeActive event (checked by the caller).
pub fn produce(g: &mut Game, c: ChangeActiveView) -> R {
    let Some(to) = c.to else { return Ok(()) };
    g.run_fx_unit(Effect::ChangeActive { p: c.p as u8, from: c.from, to, change: c.change, cause: c.cause })
}

/// The rule's promotion of the Benched Pokémon in `to` to `p`'s empty Active Spot.
pub fn promote(g: &mut Game, p: usize, to: SlotId) -> R<bool> {
    let cause = Cause::rule(crate::cause::RuleWhich::Promotion, p as u8);
    change_active(g, ChangeActiveView::of(g, p, Some(to), ActiveChange::Promotion, cause))
}

// ---------------------------------------------------------------------------
// Consequences, applied by the event's reducer

/// What the change does (events design 4.4): the Pokémon leaving the Active Spot loses its Special Conditions
/// (RemoveCondition by the change's cause; a consequence of the rule, never refused) and every effect on it from
/// attacks (APR A-03, C-03..C-05; id891: all attack effects that don't place a marker; id356: the effect was on that
/// Pokémon and goes with it to the Bench); the Pokémon moving to the Active Spot keeps whatever it has. The spots
/// change places, and each Pokémon's card records that it moved this turn (APR E-25: card-bound,
/// `enter::IDENTITY`).
pub fn reducer(g: &mut Game, id: EffId) -> R {
    let Effect::ChangeActive { p, from, to, cause, .. } = *g.e(id) else { return Ok(()) };
    let p = p as usize;
    let Some(bi) = g.st.players[p].bench_index_of(to) else { crate::bail!("INVALID_TARGET") };
    let leaving = g.st.slot_pokemon(p, from);
    if leaving.is_some() {
        crate::engine::condition::recover_by_rule(g, SlotRef::new(p, from), cause, &[])?;
        let pl = &mut g.st.players[p];
        // The player's markers about the Active Pokémon go with it.
        pl.marker.items.retain(|m| !(m.target_scope == TargetScope::Pokemon || m.name == KNOCKOUT_MARKER || m.name == CLEAR_KNOCKOUT_MARKER));
        crate::engine::game_effect::remove_attack_effects(&mut pl.slots[from as usize]);
        crate::engine::game_effect::clear_effects(&mut pl.slots[from as usize]);
    }
    let pl = &mut g.st.players[p];
    pl.active = to;
    pl.bench.as_mut_slice()[bi] = from;
    touch();
    if let Some(c) = g.st.slot_pokemon(p, to) {
        if !g.st.players[p].moved_to_active_this_turn.contains(&c) {
            g.st.players[p].moved_to_active_this_turn.push(c);
        }
        g.st.cards[c as usize].moved_to_active_this_turn = true;
    }
    if let Some(c) = leaving {
        if !g.st.players[p].moved_from_active_to_bench_this_turn.contains(&c) {
            g.st.players[p].moved_from_active_to_bench_this_turn.push(c);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! The routine against hand-built boards.
    use super::*;
    use crate::cause::{CauseKind, RuleWhich};
    use serde_json::json;

    const SNORLAX: &str = "Hop's Snorlax JTG 117";
    const SHUPPET: &str = "Shuppet PBL 33";
    const MIST: &str = "Mist Energy TEF 161";

    fn game(sc: serde_json::Value) -> Game {
        let mut names: Vec<&str> = Vec::new();
        for (n, k) in [(SNORLAX, 4), (SHUPPET, 4), (MIST, 4)] {
            names.extend(std::iter::repeat(n).take(k));
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

    fn bench(g: &Game, p: usize, i: usize) -> SlotId {
        g.st.players[p].bench.as_slice()[i]
    }

    fn attack(p: usize) -> Cause {
        Cause::new(CauseKind::Attack, None, p as u8)
    }

    /// Is the change of `p`'s Active Pokémon with `to` refused?
    fn is_refused(g: &mut Game, p: usize, to: Option<SlotId>, change: ActiveChange, cause: Cause) -> bool {
        let c = ChangeActiveView::of(g, p, to, change, cause);
        refused(g, &c).unwrap()
    }

    /// Change `p`'s Active Pokémon with `to`: did it happen?
    fn change(g: &mut Game, p: usize, to: SlotId, change: ActiveChange, cause: Cause) -> bool {
        let c = ChangeActiveView::of(g, p, Some(to), change, cause);
        change_active(g, c).unwrap()
    }

    /// The Pokémon leaving the Active Spot loses its Special Conditions and attack effects; the new one keeps its own
    /// (id356, id891).
    #[test]
    fn leaving_pokemon_loses_its_effects_the_new_one_keeps_them() {
        let mut g = game(json!({"me": {"reset": true, "active": SNORLAX, "active_conditions": ["POISONED"], "bench": [{"card": SNORLAX}]}, "opp": {"reset": true, "active": SNORLAX}}));
        let me = g.st.active_player as usize;
        let old = g.st.players[me].active;
        let b = bench(&g, me, 0);
        g.st.players[me].slots[old as usize].cannot_attack_next_turn = true;
        g.st.players[me].slots[b as usize].healed_this_turn = true;
        g.st.players[me].slots[b as usize].cannot_attack_next_turn = true;
        assert!(change(&mut g, me, b, ActiveChange::Retreat, Cause::rule(RuleWhich::Retreat, me as u8)));
        assert_eq!(g.st.players[me].active, b);
        assert!(g.st.slot(me, old).special_conditions.is_empty());
        assert!(!g.st.slot(me, old).cannot_attack_next_turn);
        assert!(g.st.slot(me, b).healed_this_turn && g.st.slot(me, b).cannot_attack_next_turn, "the new Active keeps its effects");
        let new = g.st.slot_pokemon(me, b).unwrap();
        assert!(g.st.players[me].moved_to_active_this_turn.contains(&new));
    }

    /// A switch-in done by an opponent's attack is an effect done to the Benched Pokémon: Mist Energy on it prevents
    /// it (APR C-05; JP Q&A Ninetales + Boss's Orders + Mist Energy); Mist Energy on the Active doesn't. A switch-out
    /// is done to the Active Pokémon (APR C-04, id2025).
    #[test]
    fn switch_in_and_switch_out_against_mist_energy() {
        let mut g = game(json!({"me": {"reset": true, "active": SNORLAX}, "opp": {"reset": true, "active": SNORLAX, "active_energy": [MIST], "bench": [{"card": SNORLAX, "energy": [MIST]}, {"card": SNORLAX}]}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let (misty, plain) = (bench(&g, o, 0), bench(&g, o, 1));
        assert!(is_refused(&mut g, o, Some(misty), ActiveChange::SwitchIn, attack(me)), "Mist Energy on the Benched Pokémon");
        assert!(!is_refused(&mut g, o, Some(misty), ActiveChange::SwitchIn, Cause::new(CauseKind::Trainer, None, me as u8)), "a Trainer's switch-in");
        assert!(is_refused(&mut g, o, None, ActiveChange::SwitchOut, attack(me)), "Mist Energy on the Active Pokémon stops a switch-out");
        assert!(change(&mut g, o, plain, ActiveChange::SwitchIn, attack(me)), "but not a switch-in of another Pokémon");
        assert_eq!(g.st.players[o].active, plain);
    }

    /// Hide 'n' Sneak prevents the opponent's Ability's switch-in of the Benched Pokémon that has it, not the
    /// switch-in of another Pokémon when it is Active (official JP Q&A, Hariyama's Heave-Ho Catcher).
    #[test]
    fn hide_n_sneak_against_an_ability_switch_in() {
        let mut g = game(json!({"me": {"reset": true, "active": SNORLAX}, "opp": {"reset": true, "active": SHUPPET, "bench": [{"card": SHUPPET}, {"card": SNORLAX}]}}));
        let me = g.st.active_player as usize;
        let o = 1 - me;
        let ability = Cause::new(CauseKind::Ability, None, me as u8);
        let (shuppet, snorlax) = (bench(&g, o, 0), bench(&g, o, 1));
        assert!(is_refused(&mut g, o, Some(shuppet), ActiveChange::SwitchIn, ability));
        assert!(change(&mut g, o, snorlax, ActiveChange::SwitchIn, ability), "the Active Shuppet doesn't stop it");
        // Its owner's own switch isn't the opponent's effect.
        assert!(change(&mut g, o, shuppet, ActiveChange::Switch, Cause::new(CauseKind::Trainer, None, o as u8)));
    }

    /// Does the predicate require a cause that isn't a game rule (an attack, an Ability, a card)?
    fn needs_non_rule(p: &EventPred) -> bool {
        fn cause(c: &CausePred) -> bool {
            match c {
                CausePred::Kind(k) => !matches!(k, CauseKind::Rule { .. }),
                // A rule's cause has no card.
                CausePred::Card(_) | CausePred::Pokemon(_) | CausePred::Attack(_) => true,
                CausePred::All(ps) => ps.iter().any(cause),
                CausePred::Any(ps) => ps.iter().all(cause),
                CausePred::Rule | CausePred::By(_) | CausePred::Not(_) => false,
            }
        }
        match p {
            EventPred::Cause(c) => cause(c),
            EventPred::All(ps) => ps.iter().any(needs_non_rule),
            EventPred::Any(ps) => ps.iter().all(needs_non_rule),
            _ => false,
        }
    }

    /// No `Prevent` over ChangeActive can match a rule's cause (a retreat, a promotion): legality doesn't read the
    /// declarations for the retreat (legal.rs `Ctx`), and the rule's promotion must happen. A declaration that could
    /// would need both readers.
    #[test]
    fn no_prevention_matches_a_rule_cause() {
        let mut n = 0;
        for s in crate::cards::registry::SPECS.iter() {
            for ps in s.passives {
                let crate::spec::passive::Modifier::Prevent(p) = &ps.modifier else { continue };
                if p.from.is_never() || !p.from.effect_kinds().has(crate::effects::k::CHANGE_ACTIVE) {
                    continue;
                }
                n += 1;
                assert!(needs_non_rule(&p.from), "{}: a Prevent over ChangeActive that a rule's cause could match", s.class);
            }
        }
        assert!(n >= 13, "the ChangeActive preventions are read ({n})");
        // The ones an attack leaves on a Pokémon too (legality doesn't read `Slot::lasting_prevents` for the retreat).
        use crate::spec::passive as ps;
        for p in [&ps::LASTING_PREVENT_EFFECTS, &ps::LASTING_PREVENT_DAMAGE, &ps::LASTING_PREVENT_DAMAGE_FROM_BASIC, &ps::LASTING_PREVENT_DAMAGE_FROM_EVOLUTION, &ps::LASTING_PREVENT_DAMAGE_FROM_ABILITY, &ps::LASTING_PREVENT_DAMAGE_FROM_BASIC_NON_COLORLESS] {
            assert!(needs_non_rule(&p.from), "a lasting Prevent a rule's cause could match");
        }
    }

    /// A promotion fills the empty Active Spot; nothing leaves.
    #[test]
    fn promotion_needs_an_empty_active_spot() {
        let mut g = game(json!({"me": {"reset": true, "active": SNORLAX, "bench": [{"card": SNORLAX}]}, "opp": {"reset": true, "active": SNORLAX}}));
        let me = g.st.active_player as usize;
        let b = bench(&g, me, 0);
        assert!(!promote(&mut g, me, b).unwrap(), "the Active Spot isn't empty");
        let a = g.st.players[me].active;
        g.st.players[me].slots[a as usize].cards = Default::default();
        assert!(promote(&mut g, me, b).unwrap());
        assert_eq!(g.st.players[me].active, b);
        assert!(g.st.slot_pokemon(me, a).is_none());
    }
}
