//! The ApplyEffect event of events batch 6 (user decision D4): an attack puts a lasting effect on a Pokémon or a player
//! ("during your opponent's next turn, ...", "this Pokémon can't attack during your next turn"). One routine for every
//! `Op::Arm` and for an attack's "it can't be used during their next turn", in the `engine::condition` pattern: the
//! checks (the target is there; the locks; the preventions: "prevent all effects of attacks done to this Pokémon" stops
//! an effect put on that Pokémon, APR C-17; id2341: Mist Energy keeps Pouncing Trap off it), then the event, whose
//! reducer writes the effect where it is kept until it ends (the lasting table, `engine::lasting`). The reader doesn't look at the effect itself: the event's target and cause decide.
//!
//! Existing effects are not removed (APR C-17; Mist Energy's and Repelling Veil's "Existing effects are not removed";
//! id2341): the reader is asked only when an effect is put, never again when a protection starts later.

use crate::cause::Cause;
use crate::effects::{ApplyTarget, EffId, Effect, SlotRef};
use crate::game::{Game, R};
use crate::list::CardId;
use crate::spec::event::{EventKind, EventView};
use crate::spec::ops::state::{LastingLockSpec, Lasting, LockOn, LockUntil, NamedAttack};
use crate::state::{AttackRef, LastingWhat};

/// The ApplyEffect event as predicates read it: done to the Pokémon (`slot`) or the player (`owner`, no spot).
pub fn apply_view(g: &Game, target: ApplyTarget, cause: Cause) -> EventView {
    let (owner, slot) = match target {
        ApplyTarget::Slot(s) => (s.p, Some(s)),
        ApplyTarget::Player(p) => (p, None),
    };
    EventView { card: slot.and_then(|s| g.st.slot_pokemon(s.p as usize, s.s)), slot, ..EventView::new(EventKind::ApplyEffect, cause, owner, crate::spec::event::whose_turn(g)) }
}

/// ApplyEffect: player `p`'s attack `attack` (used by `card`) puts `effect` on `target`. It doesn't happen (`Ok(false)`)
/// when the target Pokémon isn't there or the event is refused.
/// (`attack`: the attack that puts it; `None` for a Trainer's effect, Iron Defender.)
pub fn apply_effect(g: &mut Game, p: usize, target: ApplyTarget, effect: Lasting, card: CardId, attack: Option<AttackRef>, cause: Cause) -> R<bool> {
    if let ApplyTarget::Slot(s) = target {
        if g.st.slot_pokemon(s.p as usize, s.s).is_none() {
            return Ok(false);
        }
    }
    let v = apply_view(g, target, cause);
    if crate::engine::condition::refused(g, &v)?.is_some() {
        return Ok(false);
    }
    g.run_fx_unit(Effect::ApplyEffect { p: p as u8, target, effect, card, attack, cause })?;
    Ok(true)
}

/// A new Pokémon of the player in `target` came into play: each "(This includes new Pokémon that come into play.)" effect
/// on that player (`Op::ArmEach`) is put on it when it matches, one ApplyEffect event each, by the cause that put the
/// effect (batch 8 C12c).
pub fn spawn_on_entering(g: &mut Game, target: SlotRef) -> R {
    let q = target.p as usize;
    for l in crate::engine::lasting::spawns(&g.st, q).iter() {
        let LastingWhat::Spawn { spec, cause, .. } = l.what else { continue };
        let cause = Cause::unpack(cause);
        if !crate::spec::value::slot_pred_m(g, l.source, target, &spec.subject)? {
            continue;
        }
        apply_effect(g, cause.player as usize, ApplyTarget::Slot(target), spec.what, l.source, cause.attack, cause)?;
    }
    Ok(())
}

/// The effect is kept where it lasts: what each `Lasting` writes.
pub fn reducer(g: &mut Game, id: EffId) -> R {
    let Effect::ApplyEffect { p, target, effect, card, attack, cause } = *g.e(id) else { return Ok(()) };
    let p = p as usize;
    let slot_of = |t: ApplyTarget| match t {
        ApplyTarget::Slot(s) => Some(s),
        ApplyTarget::Player(_) => None,
    };
    let player_of = |t: ApplyTarget| match t {
        ApplyTarget::Slot(s) => s.p as usize,
        ApplyTarget::Player(q) => q as usize,
    };
    let q = player_of(target);
    if let Lasting::Lock(spec) = effect {
        let attack_named = match spec.attack {
            NamedAttack::Any => None,
            NamedAttack::This => attack,
            NamedAttack::Chosen(a) => Some(a),
        };
        // The turns of the locked player it lasts: through the attacker's next turn (its own Pokémon: this turn ends
        // first), through the opponent's next turn, or until the Pokémon's effects end.
        let turns = match spec.until {
            LockUntil::YourNextTurn => 2,
            LockUntil::OpponentsNextTurn => 1,
            LockUntil::LeavesActive => LastingWhat::UNTIL_REMOVED,
        };
        let source = cause.card.or(attack.map(|a| a.card)).unwrap_or(card);
        match slot_of(target) {
            Some(t) => crate::engine::lasting::add_pokemon_lock(&mut g.st, t.p as usize, t.s, spec.lock, turns, source, attack_named),
            None => crate::engine::lasting::add_player_lock(&mut g.st, q, spec.lock, turns, source),
        }
        return Ok(());
    }
    if let Some(t) = slot_of(target) {
        let (tp, ts) = (t.p as usize, t.s);
        let st = &mut g.st;
        use crate::engine::lasting as l;
        match effect {
            Lasting::TakesLessDamage(n, from) => l::set_takes_less(st, tp, ts, n, from, card),
            Lasting::DealsLessDamage(n) => l::set_deals_less(st, tp, ts, n, card),
            Lasting::TakesMoreDamage(n) => l::set_takes_more(st, tp, ts, n, p as u8, card),
            Lasting::PreventDamage(src) => l::add_prevent(st, tp, ts, crate::spec::passive::lasting_index(src.spec()), cause.card.or(attack.map(|a| a.card)).unwrap_or(card)),
            Lasting::PreventAttackEffects => l::add_prevent(st, tp, ts, crate::spec::passive::lasting_index(&crate::spec::passive::LASTING_PREVENT_EFFECTS), cause.card.or(attack.map(|a| a.card)).unwrap_or(card)),
            // The two lasting triggers (they fire in the Damage routine's step-7 window and in the KnockOut routine): an
            // attack's only.
            Lasting::Retaliate(amount) => {
                if let Some(attack) = attack {
                    l::set_retaliate(st, tp, ts, amount, attack, p as u8, card)
                }
            }
            Lasting::DiscardAttackerEnergyIfKnockedOut => {
                if let Some(attack) = attack {
                    l::set_discard_attacker_energy(st, tp, ts, attack, p as u8, card)
                }
            }
            Lasting::IncreaseAttackCost => l::set_cost_increase(st, tp, ts, false, p as u8, card),
            Lasting::IncreaseRetreatCost => l::set_cost_increase(st, tp, ts, true, p as u8, card),
            Lasting::NoWeakness => l::set_no_weakness(st, tp, ts, card),
            Lasting::Lock(_) => {}
        }
    }
    Ok(())
}

/// The target of each lasting effect of `p`'s attack from `source`: the opponent's Active Pokémon, the attacking
/// Pokémon, its player's Active Pokémon, or the opponent.
pub fn target_of(g: &Game, p: usize, source: SlotRef, effect: Lasting) -> ApplyTarget {
    let o = 1 - p;
    match effect {
        Lasting::Lock(LastingLockSpec { on: LockOn::DefendingPokemon, .. }) | Lasting::DealsLessDamage(_) | Lasting::TakesMoreDamage(_) | Lasting::IncreaseAttackCost | Lasting::IncreaseRetreatCost => {
            ApplyTarget::Slot(SlotRef::new(o, g.st.players[o].active))
        }
        Lasting::Lock(LastingLockSpec { on: LockOn::ThisPokemon, .. }) => ApplyTarget::Slot(source),
        Lasting::Lock(LastingLockSpec { on: LockOn::Opponent, .. }) => ApplyTarget::Player(o as u8),
        _ => ApplyTarget::Slot(SlotRef::new(p, g.st.players[p].active)),
    }
}
