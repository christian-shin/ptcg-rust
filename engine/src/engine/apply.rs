//! The ApplyEffect event of events batch 6 (user decision D4): an attack puts a lasting effect on a Pokémon or a player
//! ("during your opponent's next turn, ...", "this Pokémon can't attack during your next turn"). One routine for every
//! `Op::Arm` and for an attack's "it can't be used during their next turn", in the `engine::condition` pattern: the
//! checks (the target is there; the locks; the preventions: "prevent all effects of attacks done to this Pokémon" stops
//! an effect put on that Pokémon, APR C-17; id2341: Mist Energy keeps Pouncing Trap off it), then the event, whose
//! reducer writes the effect where it is kept until it ends (the Pokémon's lasting fields, `Slot::lasting_prevents`, the
//! player's lasting locks). The reader doesn't look at the effect itself: the event's target and cause decide.
//!
//! Existing effects are not removed (APR C-17; Mist Energy's and Repelling Veil's "Existing effects are not removed";
//! id2341): the reader is asked only when an effect is put, never again when a protection starts later.

use crate::cause::Cause;
use crate::effects::{ApplyTarget, EffId, Effect, SlotRef};
use crate::game::{Game, R};
use crate::list::CardId;
use crate::spec::event::{EventKind, EventView};
use crate::spec::ops::state::Lasting;
use crate::state::{AttackRef, StoredRetaliate};

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
pub fn apply_effect(g: &mut Game, p: usize, target: ApplyTarget, effect: Lasting, card: CardId, attack: AttackRef, cause: Cause) -> R<bool> {
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

/// The effect is kept where it lasts: what each `Lasting` writes.
pub fn reducer(g: &mut Game, id: EffId) -> R {
    let Effect::ApplyEffect { p, target, effect, card, attack, cause } = *g.e(id) else { return Ok(()) };
    let p = p as usize;
    let name = g.st.cdef(attack.card).attacks[attack.idx()].name;
    let slot_of = |t: ApplyTarget| match t {
        ApplyTarget::Slot(s) => Some(s),
        ApplyTarget::Player(_) => None,
    };
    let player_of = |t: ApplyTarget| match t {
        ApplyTarget::Slot(s) => s.p as usize,
        ApplyTarget::Player(q) => q as usize,
    };
    let q = player_of(target);
    if let Some(t) = slot_of(target) {
        let slot = &mut g.st.players[t.p as usize].slots[t.s as usize];
        match effect {
            Lasting::PreventRetreat => slot.cannot_retreat_next_turn = true,
            Lasting::CannotAttackNextTurn => slot.cannot_attack_next_turn_pending = true,
            Lasting::CannotUseThisAttackNextTurn => {
                if !slot.cannot_use_attacks_next_turn_pending.contains(&name) {
                    slot.cannot_use_attacks_next_turn_pending.push(name);
                }
            }
            Lasting::CannotUseAttack(n) => slot.blocked_attack_name_next_turn = Some(n),
            Lasting::BlockThisAttackUntilLeavesActive => slot.blocked_attack_name_until_leaves_active = Some(name),
            Lasting::TakesLessDamage(n) => slot.damage_reduction_next_turn = n,
            Lasting::DealsLessDamage(n) => slot.attack_damage_reduction_next_turn = n.max(0),
            Lasting::TakesMoreDamage(n) => {
                let already = slot.defending_extra_damage_next_turn > 0 && !slot.defending_extra_damage_pending && slot.defending_extra_damage_attacker == Some(p as u8);
                slot.defending_extra_damage_next_turn = n;
                slot.defending_extra_damage_attacker = Some(p as u8);
                if already {
                    slot.defending_extra_damage_rearm_after_attack = true;
                } else {
                    slot.defending_extra_damage_pending = true;
                }
            }
            Lasting::PreventDamage(src) => push_prevent(slot, src.spec(), cause, attack),
            Lasting::PreventAttackEffects => push_prevent(slot, &crate::spec::passive::LASTING_PREVENT_EFFECTS, cause, attack),
            Lasting::Retaliate(n) => slot.retaliate_on_damage_next_turn_pending = Some(StoredRetaliate { damage: n, attack, source_card: card, attacker: p as u8 }),
            Lasting::DiscardAttackerEnergyIfKnockedOut => {
                slot.discard_attacker_energy_if_ko_next_turn = true;
                slot.discard_attacker_energy_if_ko_next_turn_pending = true;
                slot.discard_attacker_energy_if_ko_attack = Some(attack);
                slot.discard_attacker_energy_if_ko_source_card = Some(card);
                slot.discard_attacker_energy_if_ko_attacker = Some(p as u8);
            }
            Lasting::IncreaseAttackCost => {
                slot.attack_cost_increase_next_turn_pending = 1;
                slot.attack_cost_increase_next_turn_attacker = Some(p as u8);
            }
            Lasting::IncreaseRetreatCost => {
                slot.retreat_cost_increase_next_turn_pending = 1;
                slot.retreat_cost_increase_next_turn_attacker = Some(p as u8);
            }
            Lasting::NoWeakness => slot.no_weakness_next_turn_pending = true,
            Lasting::SelfCannotRetreat => slot.cannot_retreat_next_turn_pending = true,
            Lasting::OppCannotPlay(_) | Lasting::OppSmallEnergyCannotAttack(_) => {}
        }
        return Ok(());
    }
    let pl = &mut g.st.players[q];
    match effect {
        Lasting::OppCannotPlay(lock) => crate::engine::phase::apply_play_lock(pl, lock, 1, cause.card.unwrap_or(attack.card)),
        Lasting::OppSmallEnergyCannotAttack(n) => {
            pl.cannot_attack_max_energy = Some(n);
            pl.cannot_attack_max_energy_turns_remaining = pl.cannot_attack_max_energy_turns_remaining.max(1);
        }
        _ => {}
    }
    Ok(())
}

/// A lasting `Prevent` on the Pokémon, pending until the end of this turn; a newer one of the same declaration replaces
/// the older. Evaluated for the attacking Pokémon (its owner's opponent is "your opponent").
fn push_prevent(slot: &mut crate::state::Slot, spec: &'static crate::spec::passive::PreventSpec, cause: Cause, attack: AttackRef) {
    let source = cause.card.unwrap_or(attack.card);
    let spec = crate::spec::passive::lasting_index(spec);
    slot.lasting_prevents.retain(|x| x.spec != spec);
    slot.lasting_prevents.push(crate::state::LastingPrevent { spec, source, pending: true });
}

/// The target of each lasting effect of `p`'s attack from `source`: the opponent's Active Pokémon, the attacking
/// Pokémon, its player's Active Pokémon, or the opponent.
pub fn target_of(g: &Game, p: usize, source: SlotRef, effect: Lasting) -> ApplyTarget {
    let o = 1 - p;
    match effect {
        Lasting::PreventRetreat | Lasting::DealsLessDamage(_) | Lasting::TakesMoreDamage(_) | Lasting::IncreaseAttackCost | Lasting::IncreaseRetreatCost | Lasting::CannotUseAttack(_) => {
            ApplyTarget::Slot(SlotRef::new(o, g.st.players[o].active))
        }
        Lasting::BlockThisAttackUntilLeavesActive => ApplyTarget::Slot(source),
        Lasting::OppCannotPlay(_) | Lasting::OppSmallEnergyCannotAttack(_) => ApplyTarget::Player(o as u8),
        _ => ApplyTarget::Slot(SlotRef::new(p, g.st.players[p].active)),
    }
}
