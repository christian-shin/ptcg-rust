//! `play-card-reducer.ts` and `player-turn-reducer.ts`, plus
//! `Player.switchPokemon`.

use crate::effects::*;
use crate::engine::game_effect::{clear_effects, remove_attack_effects};
use crate::game::{Action, Game, R};
use crate::list::*;
use crate::markers::*;
use crate::prompts::get_target;
use crate::state::*;
use crate::types::*;

/// `Player.switchPokemon(target, store, state)`.
pub fn switch_pokemon(g: &mut Game, p: usize, target: SlotId) -> R {
    switch_pokemon_ex(g, p, target, true)
}

/// `Player.switchPokemon(target)` called without `store, state`: the same
/// board changes, but no MovedToActiveEffect / MovedFromActiveToBenchEffect
/// is dispatched (so e.g. ability-lock activation orders are not touched).
pub fn switch_pokemon_silent(g: &mut Game, p: usize, target: SlotId) -> R {
    switch_pokemon_ex(g, p, target, false)
}

fn switch_pokemon_ex(g: &mut Game, p: usize, target: SlotId, dispatch: bool) -> R {
    let bi = match g.st.players[p].bench_index_of(target) {
        Some(i) => i,
        None => return Ok(()),
    };
    let benched_out = g.st.active_pokemon(p);
    let pl = &mut g.st.players[p];
    pl.marker.items.retain(|m| !(m.target_scope == TargetScope::Pokemon || m.name == KNOCKOUT_MARKER || m.name == CLEAR_KNOCKOUT_MARKER));
    let old = pl.active;
    remove_attack_effects(&mut pl.slots[old as usize]);
    clear_effects(&mut pl.slots[old as usize]);
    pl.slots[old as usize].special_conditions.clear();
    pl.active = pl.bench.as_slice()[bi];
    pl.bench.as_mut_slice()[bi] = old;
    touch();
    let new_active = g.st.players[p].active;
    if let Some(c) = g.st.slot_pokemon(p, new_active) {
        if !g.st.players[p].moved_to_active_this_turn.contains(&c) {
            g.st.players[p].moved_to_active_this_turn.push(c);
        }
        g.st.cards[c as usize].moved_to_active_this_turn = true;
        if dispatch {
            g.run_fx_unit(Effect::MovedToActive { p: p as u8, card: c })?;
        }
    }
    if let Some(c) = benched_out {
        if !g.st.players[p].moved_from_active_to_bench_this_turn.contains(&c) {
            g.st.players[p].moved_from_active_to_bench_this_turn.push(c);
        }
        if dispatch {
            g.run_fx_unit(Effect::MovedFromActiveToBench { p: p as u8, card: c })?;
        }
    }
    // The Active Spot changed (a silent switch dispatches nothing): locks may take hold or let go.
    crate::spec::passive::lock_sync(g);
    Ok(())
}

fn find_pokemon_target(g: &Game, p: usize, t: CardTarget) -> Option<SlotRef> {
    get_target(&g.st, p, t).ok()
}

/// Check only: may this Energy be attached from the hand to `target` now?
/// Returns the slot and whether the attachment uses the turn's one attachment
/// (not with Dragon's Wish or unlimited attachments).
pub fn can_attach_energy(g: &Game, p: usize, target: CardTarget) -> R<(SlotRef, bool)> {
    let t = match find_pokemon_target(g, p, target) {
        Some(t) if !g.st.slot(t.p as usize, t.s).cards.is_empty() => t,
        _ => crate::bail!("INVALID_TARGET"),
    };
    let pl = &g.st.players[p];
    if pl.used_dragons_wish || g.st.rules.unlimited_energy_attachments {
        return Ok((t, false));
    }
    if pl.energy_played_turn == g.st.turn {
        crate::bail!("ENERGY_ALREADY_ATTACHED");
    }
    Ok((t, true))
}

/// Check only: the turn rules for playing this Supporter (first turn, one per turn).
pub fn can_play_supporter_card(g: &Game, p: usize, card: CardId) -> R {
    if g.st.turn == 1 && !g.st.cdef(card).first_turn {
        crate::bail!("CANNOT_PLAY_THIS_CARD");
    }
    if !g.st.players[p].supporter.is_empty() {
        crate::bail!("SUPPORTER_ALREADY_PLAYED");
    }
    Ok(())
}

/// Check only: the turn rules for playing this Stadium (one per turn, not the one in play).
pub fn can_play_stadium_card(g: &Game, p: usize, card: CardId) -> R {
    let d = g.st.cdef(card);
    let stadium = g.st.stadium_card();
    let hyperrogue = d.name == "Hyperrogue Ange Floette" && stadium.map(|s| g.st.cdef(s).name == "Prism Tower").unwrap_or(false);
    if g.st.players[p].stadium_played_turn == g.st.turn && !hyperrogue {
        crate::bail!("STADIUM_ALREADY_PLAYED");
    }
    if let Some(s) = stadium {
        if g.st.cdef(s).name == d.name {
            crate::bail!("SAME_STADIUM_ALREADY_IN_PLAY");
        }
    }
    Ok(())
}

/// Check only: a Tool needs a Pokémon target.
pub fn can_play_tool_card(target: Option<SlotRef>) -> R<SlotRef> {
    match target {
        Some(t) => Ok(t),
        None => crate::bail!("INVALID_TARGET"),
    }
}

/// Check only: using the Stadium in play (once per turn, needs one).
pub fn can_use_stadium(g: &Game, p: usize) -> R<CardId> {
    if g.st.players[p].stadium_used_turn == g.st.turn {
        crate::bail!("STADIUM_ALREADY_USED");
    }
    match g.st.stadium_card() {
        Some(s) => Ok(s),
        None => crate::bail!("NO_STADIUM_IN_PLAY"),
    }
}

/// Check only: the core rule for using this Ability from this zone (the
/// printed use-from flags). The caller found the power with the
/// `CheckPokemonPowers` read.
pub fn can_use_ability_core(g: &Game, power: PowerRef, zone: SlotType) -> R {
    let pd = &g.st.cdef(power.card).powers[power.index as usize];
    match zone {
        SlotType::Active | SlotType::Bench if !pd.use_when_in_play => crate::bail!("CANNOT_USE_POWER"),
        SlotType::Hand if !pd.use_from_hand => crate::bail!("CANNOT_USE_POWER"),
        SlotType::Discard if !pd.use_from_discard => crate::bail!("CANNOT_USE_POWER"),
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
    let card = match g.st.players[p].hand.get(hand_index as usize) {
        Some(c) => c,
        None => crate::bail!("UNKNOWN_CARD"),
    };
    let d = g.st.cdef(card);
    if d.is_energy() {
        let (t, uses_turn_attach) = can_attach_energy(g, p, target)?;
        if uses_turn_attach {
            g.st.players[p].energy_played_turn = g.st.turn;
        }
        g.run_fx_unit(Effect::AttachEnergy { p: p as u8, card, target: t })?;
        return Ok(());
    }
    if d.is_pokemon() {
        let t = match find_pokemon_target(g, p, target) {
            Some(t) => t,
            None => crate::bail!("INVALID_TARGET"),
        };
        // useFromHandToBench / Dual Legend: no pool card uses them.
        g.run_fx_unit(Effect::PlayPokemon { p: p as u8, card, target: t, slot: target.slot, index: target.index })?;
        return Ok(());
    }
    if d.is_trainer() {
        let t = find_pokemon_target(g, p, target);
        let e = match d.trainer_type() {
            TrainerType::Supporter => {
                can_play_supporter_card(g, p, card)?;
                Effect::PlaySupporter { p: p as u8, card, target: t }
            }
            TrainerType::Stadium => {
                can_play_stadium_card(g, p, card)?;
                g.st.players[p].stadium_played_turn = g.st.turn;
                Effect::PlayStadium { p: p as u8, card }
            }
            TrainerType::Tool => Effect::AttachPokemonTool { p: p as u8, card, target: can_play_tool_card(t)? },
            TrainerType::Item => Effect::PlayItem { p: p as u8, card, target: t },
        };
        g.run_fx_unit(e)?;
        return Ok(());
    }
    g.move_card_to(ListRef::Hand(p as u8), card, ListRef::Supporter(p as u8));
    Ok(())
}

/// The checked read of the attacks the active player may use (`CheckPokemonAttacks`): the attack list and the
/// ones copied from a Benched Pokémon (`copiedAttacks`).
pub type CheckedAttacks = (SVec<AttackRef, 32>, SVec<AttackRef, 32>);

pub fn read_attack_list(g: &mut Game, p: usize) -> R<CheckedAttacks> {
    let (e, _) = g.run_fx(check_attacks_effect(g, p))?;
    Ok(match e {
        Effect::CheckPokemonAttacks { attacks, copied, .. } => (attacks, copied),
        _ => (SVec::new(), SVec::new()),
    })
}

/// Attacks available to the active player, as the AttackAction reducer builds them, from the read
/// [`read_attack_list`] made: the Active Pokémon's printed attacks, the use-on-Bench attacks of the Benched
/// ones, then the checked list. The bool marks an attack copied from a Benched Pokémon.
pub fn assemble_available_attacks(g: &Game, p: usize, checked: &CheckedAttacks) -> SVec<(AttackRef, bool), 64> {
    let mut out: SVec<(AttackRef, bool), 64> = SVec::new();
    if let Some(c) = g.st.active_pokemon(p) {
        for i in 0..g.st.cdef(c).attacks.len() {
            out.push((AttackRef { card: c, index: i as u8 }, false));
        }
    }
    let (attacks, copied) = checked;
    for &b in g.st.players[p].bench.iter() {
        if let Some(c) = g.st.slot_pokemon(p, b) {
            let d = g.st.cdef(c);
            if d.attacks.iter().any(|a| a.use_on_bench) {
                for (i, a) in d.attacks.iter().enumerate() {
                    if a.use_on_bench {
                        out.push((AttackRef { card: c, index: i as u8 }, false));
                    }
                }
                for a in attacks.iter() {
                    out.push((*a, copied.iter().any(|c| c == a)));
                }
            }
        }
    }
    for a in attacks.iter() {
        out.push((*a, copied.iter().any(|c| c == a)));
    }
    out
}

/// Attacks available to the active player (the read, then the list).
pub fn available_attacks(g: &mut Game, p: usize) -> R<SVec<(AttackRef, bool), 64>> {
    let checked = read_attack_list(g, p)?;
    Ok(assemble_available_attacks(g, p, &checked))
}

/// The attack named `name` (from the Benched Pokémon `from`, when copied by Memory Helix) in the list, as the
/// `AttackAction` reducer finds it.
pub fn find_attack(g: &Game, attacks: &[(AttackRef, bool)], name: &'static str, from: Option<&'static str>) -> Option<(AttackRef, bool)> {
    attacks
        .iter()
        .find(|r| {
            g.st.cdef(r.0.card).attacks[r.0.index as usize].name == name
                && match from {
                    Some(f) => r.1 && g.st.cdef(r.0.card).full_name == f,
                    None => true,
                }
        })
        .copied()
}

/// `new CheckPokemonAttacksEffect(player)`: seeded with the active tool's attacks.
pub fn check_attacks_effect(g: &Game, p: usize) -> Effect {
    let mut attacks = SVec::new();
    let a = g.st.players[p].active;
    if let Some(t) = g.st.slot(p, a).tools.get(0) {
        let d = g.st.cdef(t);
        if d.is_trainer() {
            for i in 0..d.attacks.len() {
                attacks.push(AttackRef { card: t, index: i as u8 });
            }
        }
    }
    Effect::CheckPokemonAttacks { p: p as u8, attacks, copied: SVec::new() }
}

/// The Pokémon card whose Ability a `UseAbility` names (`None`: nothing to use).
pub fn ability_source(g: &Game, p: usize, target: CardTarget) -> R<Option<CardId>> {
    Ok(match target.slot {
        SlotType::Active | SlotType::Bench => {
            let t = get_target(&g.st, p, target)?;
            g.st.slot_pokemon(t.p as usize, t.s)
        }
        SlotType::Discard => g.st.players[p].discard.get(target.index as usize).filter(|c| g.st.cdef(*c).is_pokemon()),
        SlotType::Hand => g.st.players[p].hand.get(target.index as usize).filter(|c| g.st.cdef(*c).is_pokemon()),
        _ => None,
    })
}

pub fn player_turn_reducer(g: &mut Game, a: Action) -> R {
    if g.st.phase != GamePhase::PlayerTurn {
        return Ok(());
    }
    let p = g.st.active_player as usize;
    match a {
        Action::Pass => {
            g.run_fx_unit(Effect::EndTurn { p: p as u8 })?;
        }
        Action::Retreat { bench_index } => {
            g.run_fx(Effect::Retreat {
                p: p as u8,
                bench_index,
                ignore_status_conditions: false,
                move_retreat_cost_to: ListRef::Discard(p as u8),
            })?;
            let act = g.st.players[p].active;
            clear_effects(&mut g.st.players[p].slots[act as usize]);
        }
        Action::Attack { name, from } => {
            let pokemon = g.st.active_pokemon(p);
            let attacks = available_attacks(g, p)?;
            // `from` names the Benched Pokemon of an attack copied by Memory Helix.
            let found = find_attack(g, attacks.as_slice(), name, from);
            let (attack, copied) = match found {
                Some(r) => r,
                None => crate::bail!("UNKNOWN_ATTACK"),
            };
            let source = SlotRef::new(p, g.st.players[p].active);
            // An attack copied from a Benched Pokémon (Mew ex Memory Helix) runs as the Active Pokémon's.
            let delegate_from = if copied { Some(attack.card) } else { None };
            g.run_fx_unit(Effect::UseAttack { p: p as u8, attack, source, ignore_status_conditions: false, barrage_used: false, delegate_from })?;
            g.st.last_attack = Some(attack);
            if let Some(pc) = pokemon {
                g.st.player_last_attack[p] = Some((attack, pc));
                g.st.player_last_attack_turn[p] = g.st.turn;
            }
        }
        Action::UseAbility { name, target } => {
            let card = ability_source(g, p, target)?;
            if let Some(c) = card {
                let mut powers = SVec::new();
                for i in 0..g.st.cdef(c).powers.len() {
                    powers.push(PowerRef { card: c, index: i as u8 });
                }
                let (e, _) = g.run_fx(Effect::CheckPokemonPowers { p: p as u8, target: c, powers })?;
                let powers = match e {
                    Effect::CheckPokemonPowers { powers, .. } => powers,
                    _ => SVec::new(),
                };
                let power = match powers.iter().find(|r| g.st.cdef(r.card).powers[r.index as usize].name == name) {
                    Some(r) => *r,
                    None => crate::bail!("UNKNOWN_POWER"),
                };
                can_use_ability_core(g, power, target.slot)?;
                g.run_fx_unit(Effect::UsePower { p: p as u8, power, card: c, target, bench_target: None })?;
            }
        }
        Action::UseTrainerAbility { .. } => {
            // Trainer powers from hand/discard: no pool card has one.
        }
        Action::UseStadium => {
            let stadium = can_use_stadium(g, p)?;
            g.run_fx_unit(Effect::UseStadium { p: p as u8, stadium })?;
        }
        Action::PlayCard { .. } => {}
    }
    Ok(())
}
