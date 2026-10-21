//! Playing cards and the actions of a turn. Changing the Active Pokémon is the ChangeActive event
//! (`engine::change_active`).

use crate::effects::*;
use crate::game::{Action, Game, R};
use crate::list::*;
use crate::prompts::get_target;
use crate::state::*;
use crate::types::*;

fn find_pokemon_target(g: &Game, p: usize, t: CardTarget) -> Option<SlotRef> {
    get_target(&g.st, p, t).ok()
}

/// Check only: may this Energy be attached from the hand to `target` now?
/// Returns the slot.
pub fn can_attach_energy(g: &Game, p: usize, target: CardTarget) -> R<SlotRef> {
    let t = match find_pokemon_target(g, p, target) {
        Some(t) if !g.st.slot(t.p as usize, t.s).cards().is_empty() => t,
        _ => crate::bail!("INVALID_TARGET"),
    };
    let pl = &g.st.players[p];
    if pl.energy_played_turn == g.st.turn {
        crate::bail!("ENERGY_ALREADY_ATTACHED");
    }
    Ok(t)
}

/// Check only: using the Stadium in play (APR B-04: once during each player's turn, a Stadium whose card says it can be
/// used: `CardSpec::use_stadium`; one without use text can't be used).
pub fn can_use_stadium(g: &Game, p: usize) -> R<CardId> {
    if g.st.players[p].stadium_used_turn == g.st.turn {
        crate::bail!("STADIUM_ALREADY_USED");
    }
    match g.st.stadium_card() {
        Some(s) if crate::cards::spec_for(g.st.cards[s as usize].def).map_or(false, |sp| sp.use_stadium.is_some()) => Ok(s),
        Some(_) => crate::bail!("CANNOT_USE_STADIUM"),
        None => crate::bail!("NO_STADIUM_IN_PLAY"),
    }
}

/// Check only: the core rule for using this Ability from this zone: an Ability of a Pokémon in play is used when it has a
/// program (a passive Ability is never used), and no Ability is used from the hand or the discard pile. The caller found the
/// power among the ones the card has now (`derived::powers`).
pub fn can_use_ability_core(g: &Game, power: PowerRef, zone: SlotType) -> R {
    match zone {
        SlotType::Active | SlotType::Bench if !crate::spec::run::power_has_program(g, power) => crate::bail!("CANNOT_USE_POWER"),
        SlotType::Hand | SlotType::Discard => crate::bail!("CANNOT_USE_POWER"),
        _ => {}
    }
    Ok(())
}

pub fn play_card_reducer(g: &mut Game, a: Action) -> R {
    let (hand_index, target) = match a {
        Action::PlayCard { hand_index, target } => (hand_index, target),
        _ => return Ok(()),
    };
    if g.st.phase != GamePhase::PlayerTurn {
        return Ok(());
    }
    let p = g.st.active_player as usize;
    let card = match g.st.players[p].hand().get(hand_index as usize) {
        Some(c) => c,
        None => crate::bail!("UNKNOWN_CARD"),
    };
    let d = g.st.cdef(card);
    if d.is_energy() {
        let t = can_attach_energy(g, p, target)?;
        // The Attach event from the hand by the rule (`manual`), with its checks.
        return crate::engine::attach::play_energy(g, p, card, t);
    }
    if d.is_pokemon() {
        let t = match find_pokemon_target(g, p, target) {
            Some(t) => t,
            None => crate::bail!("INVALID_TARGET"),
        };
        return crate::engine::enter::play_from_hand(g, p, card, t);
    }
    if d.is_trainer() {
        // The PlayTrainer event (`engine::play_trainer`): its checks, then the play.
        let t = find_pokemon_target(g, p, target);
        return crate::engine::play_trainer::play_from_hand(g, p, card, t);
    }
    g.move_card_to(ListRef::Hand(p as u8), card, ListRef::Supporter(p as u8));
    Ok(())
}

/// The attacks the active player may use (`derived::attacks`): the attack list and the ones copied from a Benched
/// Pokémon.
pub type CheckedAttacks = (SVec<AttackRef, 32>, SVec<AttackRef, 32>);

/// Attacks available to the active player, as the AttackAction reducer builds them, from the read
/// [`read_attack_list`] made: the Active Pokémon's printed attacks, then the checked list. The bool marks an attack copied from a Benched Pokémon.
pub fn assemble_available_attacks(g: &Game, p: usize, checked: &CheckedAttacks) -> SVec<(AttackRef, bool), 64> {
    let mut out: SVec<(AttackRef, bool), 64> = SVec::new();
    if let Some(c) = g.st.active_pokemon(p) {
        for i in 0..g.st.cdef(c).attacks.len() {
            out.push((AttackRef { card: c, index: i as u8 }, false));
        }
    }
    let (attacks, copied) = checked;
    for a in attacks.iter() {
        out.push((*a, copied.iter().any(|c| c == a)));
    }
    out
}

/// Attacks available to the active player (the read, then the list).
pub fn available_attacks(g: &mut Game, p: usize) -> R<SVec<(AttackRef, bool), 64>> {
    let checked = crate::derived::attacks(g, p)?;
    Ok(assemble_available_attacks(g, p, &checked))
}

/// Is `attack` (copied from a Benched Pokémon or not) in the list of available attacks?
pub fn offers(attacks: &[(AttackRef, bool)], attack: AttackRef, copied: bool) -> bool {
    attacks.iter().any(|r| *r == (attack, copied))
}

/// The printed attack list the derived attack list starts from: seeded with the active tool's attacks.
pub fn printed_attack_list(g: &Game, p: usize) -> CheckedAttacks {
    let mut attacks = SVec::new();
    let a = g.st.players[p].active;
    if let Some(t) = g.st.slot(p, a).tools().get(0) {
        let d = g.st.cdef(t);
        if d.is_trainer() {
            for i in 0..d.attacks.len() {
                attacks.push(AttackRef { card: t, index: i as u8 });
            }
        }
    }
    (attacks, SVec::new())
}

/// The Pokémon card whose Ability a `UseAbility` names (`None`: nothing to use).
pub fn ability_source(g: &Game, p: usize, target: CardTarget) -> R<Option<CardId>> {
    Ok(match target.slot {
        SlotType::Active | SlotType::Bench => {
            let t = get_target(&g.st, p, target)?;
            g.st.slot_pokemon(t.p as usize, t.s)
        }
        SlotType::Discard => g.st.players[p].discard().get(target.index as usize).filter(|c| g.st.cdef(*c).is_pokemon()),
        SlotType::Hand => g.st.players[p].hand().get(target.index as usize).filter(|c| g.st.cdef(*c).is_pokemon()),
        _ => None,
    })
}

/// Player `p`'s Active Pokémon uses `attack`, one of its available attacks (`copied`: a Benched Pokémon's, used as the
/// Active Pokémon's own, Mew ex's Memory Helix: `attack.card` is that Benched Pokémon): the UseAttack event, then the
/// records of the attack used (the last attack, of the player and of the Pokémon). `what_if`: a host's what-if use on a
/// copy of the game (`what_if::attack`): the event's checks are not made and the attack stops before its Knock Out check.
pub fn use_attack(g: &mut Game, p: usize, attack: AttackRef, copied: bool, what_if: bool) -> R {
    let pokemon = g.st.active_pokemon(p);
    let source = SlotRef::new(p, g.st.players[p].active);
    let delegate_from = if copied { Some(attack.card) } else { None };
    g.run_fx_unit(Effect::UseAttack { p: p as u8, attack, source, ignore_status_conditions: false, second_attack: false, delegate_from, what_if })?;
    g.st.last_attack = Some(attack);
    if let Some(pc) = pokemon {
        g.st.player_last_attack[p] = Some((attack, pc));
        g.st.player_last_attack_turn[p] = g.st.turn;
    }
    Ok(())
}

pub fn player_turn_reducer(g: &mut Game, a: Action) -> R {
    if g.st.phase != GamePhase::PlayerTurn {
        return Ok(());
    }
    let p = g.st.active_player as usize;
    match a {
        Action::Pass => {
            crate::engine::phase::end_turn_event(g, p)?;
        }
        Action::Retreat => {
            g.run_fx(Effect::Retreat {
                p: p as u8,
                ignore_status_conditions: false,
                move_retreat_cost_to: ListRef::Discard(p as u8),
            })?;
            // Only the Pokémon moving to the Bench loses its effects; the new Active Pokémon keeps its own (id356, id891:
            // ChangeActive's consequences).
        }
        Action::Attack { attack, copied } => {
            let attacks = available_attacks(g, p)?;
            if !offers(attacks.as_slice(), attack, copied) {
                crate::bail!("UNKNOWN_ATTACK");
            }
            use_attack(g, p, attack, copied, false)?;
        }
        Action::UseAbility { power, target } => {
            let card = ability_source(g, p, target)?;
            if let Some(c) = card {
                let powers = crate::derived::powers(g, c)?;
                if !powers.iter().any(|r| *r == power) {
                    crate::bail!("UNKNOWN_POWER");
                }
                // The UseAbility event's checks (`engine::turn_action::ability_checks`, the function legality calls).
                if let Some(code) = crate::engine::turn_action::ability_checks(g, p, power, c, target.slot)? {
                    crate::bail!(code);
                }
                g.run_fx_unit(Effect::UsePower { p: p as u8, power, card: c, target, bench_target: None })?;
            }
        }
        Action::UseStadium => {
            // The UseStadium event's checks (`engine::turn_action::stadium_checks`, the function legality calls).
            let stadium = match crate::engine::turn_action::stadium_checks(g, p)? {
                Ok(s) => s,
                Err(code) => crate::bail!(code),
            };
            g.run_fx_unit(Effect::UseStadium { p: p as u8, stadium })?;
        }
        Action::PlayCard { .. } => {}
    }
    Ok(())
}
