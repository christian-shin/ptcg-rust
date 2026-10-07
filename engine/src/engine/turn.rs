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
    let new_active = g.st.players[p].active;
    if let Some(c) = g.st.slot_pokemon(p, new_active) {
        if !g.st.players[p].moved_to_active_this_turn.contains(&c) {
            g.st.players[p].moved_to_active_this_turn.push(c);
        }
        g.st.cards[c as usize].moved_to_active_this_turn = true;
        if dispatch {
            g.run_fx(Effect::MovedToActive { p: p as u8, card: c })?;
        }
    }
    if let Some(c) = benched_out {
        if !g.st.players[p].moved_from_active_to_bench_this_turn.contains(&c) {
            g.st.players[p].moved_from_active_to_bench_this_turn.push(c);
        }
        if dispatch {
            g.run_fx(Effect::MovedFromActiveToBench { p: p as u8, card: c })?;
        }
    }
    Ok(())
}

fn find_pokemon_target(g: &Game, p: usize, t: CardTarget) -> Option<SlotRef> {
    get_target(&g.st, p, t).ok()
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
        let t = match find_pokemon_target(g, p, target) {
            Some(t) if !g.st.slot(t.p as usize, t.s).cards.is_empty() => t,
            _ => crate::bail!("INVALID_TARGET"),
        };
        let pl = &mut g.st.players[p];
        if pl.used_dragons_wish || g.st.rules.unlimited_energy_attachments {
            g.run_fx(Effect::AttachEnergy { p: p as u8, card, target: t })?;
            return Ok(());
        }
        if pl.energy_played_turn == g.st.turn {
            crate::bail!("ENERGY_ALREADY_ATTACHED");
        }
        pl.energy_played_turn = g.st.turn;
        g.run_fx(Effect::AttachEnergy { p: p as u8, card, target: t })?;
        return Ok(());
    }
    if d.is_pokemon() {
        let t = match find_pokemon_target(g, p, target) {
            Some(t) => t,
            None => crate::bail!("INVALID_TARGET"),
        };
        // useFromHandToBench / Dual Legend: no pool card uses them.
        g.run_fx(Effect::PlayPokemon { p: p as u8, card, target: t, slot: target.slot, index: target.index })?;
        return Ok(());
    }
    if d.is_trainer() {
        let t = find_pokemon_target(g, p, target);
        let e = match d.trainer_type() {
            TrainerType::Supporter => {
                if g.st.turn == 1 && !d.first_turn {
                    crate::bail!("CANNOT_PLAY_THIS_CARD");
                }
                if !g.st.players[p].supporter.is_empty() {
                    crate::bail!("SUPPORTER_ALREADY_PLAYED");
                }
                Effect::PlaySupporter { p: p as u8, card, target: t }
            }
            TrainerType::Stadium => {
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
                g.st.players[p].stadium_played_turn = g.st.turn;
                Effect::PlayStadium { p: p as u8, card }
            }
            TrainerType::Tool => match t {
                Some(t) => Effect::AttachPokemonTool { p: p as u8, card, target: t },
                None => crate::bail!("INVALID_TARGET"),
            },
            TrainerType::Item => Effect::PlayItem { p: p as u8, card, target: t },
        };
        g.run_fx(e)?;
        return Ok(());
    }
    g.move_card_to(ListRef::Hand(p as u8), card, ListRef::Supporter(p as u8));
    Ok(())
}

/// Attacks available to the active player, as the AttackAction reducer builds them.
/// The bool marks an attack copied from a Benched Pokémon (`CheckPokemonAttacksEffect.copiedAttacks`).
pub fn available_attacks(g: &mut Game, p: usize) -> R<SVec<(AttackRef, bool), 64>> {
    let mut out: SVec<(AttackRef, bool), 64> = SVec::new();
    if let Some(c) = g.st.active_pokemon(p) {
        for i in 0..g.st.cdef(c).attacks.len() {
            out.push((AttackRef { card: c, index: i as u8 }, false));
        }
    }
    let bench: Vec<SlotId> = g.st.players[p].bench.iter().copied().collect();
    for b in bench {
        if let Some(c) = g.st.slot_pokemon(p, b) {
            let d = g.st.cdef(c);
            if d.attacks.iter().any(|a| a.use_on_bench) {
                for (i, a) in d.attacks.iter().enumerate() {
                    if a.use_on_bench {
                        out.push((AttackRef { card: c, index: i as u8 }, false));
                    }
                }
                let (e, _) = g.run_fx(check_attacks_effect(g, p))?;
                if let Effect::CheckPokemonAttacks { attacks, copied, .. } = e {
                    for a in attacks.iter() {
                        out.push((*a, copied.iter().any(|c| c == a)));
                    }
                }
            }
        }
    }
    let (e, _) = g.run_fx(check_attacks_effect(g, p))?;
    if let Effect::CheckPokemonAttacks { attacks, copied, .. } = e {
        for a in attacks.iter() {
            out.push((*a, copied.iter().any(|c| c == a)));
        }
    }
    Ok(out)
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

pub fn player_turn_reducer(g: &mut Game, a: Action) -> R {
    if g.st.phase != GamePhase::PlayerTurn {
        return Ok(());
    }
    let p = g.st.active_player as usize;
    match a {
        Action::Pass => {
            g.run_fx(Effect::EndTurn { p: p as u8 })?;
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
        Action::Attack { name } => {
            let pokemon = g.st.active_pokemon(p);
            let attacks = available_attacks(g, p)?;
            let (attack, copied) = match attacks.iter().find(|r| g.st.cdef(r.0.card).attacks[r.0.index as usize].name == name) {
                Some(r) => *r,
                None => crate::bail!("UNKNOWN_ATTACK"),
            };
            let source = SlotRef::new(p, g.st.players[p].active);
            // An attack copied from a Benched Pokémon (Mew ex Memory Helix) runs as the Active Pokémon's.
            let delegate_from = if copied { Some(attack.card) } else { None };
            g.run_fx(Effect::UseAttack { p: p as u8, attack, source, ignore_status_conditions: false, barrage_used: false, delegate_from })?;
            g.st.last_attack = Some(attack);
            if let Some(pc) = pokemon {
                g.st.player_last_attack[p] = Some((attack, pc));
                g.st.player_last_attack_turn[p] = g.st.turn;
            }
        }
        Action::UseAbility { name, target } => {
            let card = match target.slot {
                SlotType::Active | SlotType::Bench => {
                    let t = get_target(&g.st, p, target)?;
                    g.st.slot_pokemon(t.p as usize, t.s)
                }
                SlotType::Discard => g.st.players[p].discard.get(target.index as usize).filter(|c| g.st.cdef(*c).is_pokemon()),
                SlotType::Hand => g.st.players[p].hand.get(target.index as usize).filter(|c| g.st.cdef(*c).is_pokemon()),
                _ => None,
            };
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
                let pd = &g.st.cdef(power.card).powers[power.index as usize];
                match target.slot {
                    SlotType::Active | SlotType::Bench if !pd.use_when_in_play => crate::bail!("CANNOT_USE_POWER"),
                    SlotType::Hand if !pd.use_from_hand => crate::bail!("CANNOT_USE_POWER"),
                    SlotType::Discard if !pd.use_from_discard => crate::bail!("CANNOT_USE_POWER"),
                    _ => {}
                }
                g.run_fx(Effect::UsePower { p: p as u8, power, card: c, target, bench_target: None })?;
            }
        }
        Action::UseTrainerAbility { .. } => {
            // Trainer powers from hand/discard: no pool card has one.
        }
        Action::UseStadium => {
            if g.st.players[p].stadium_used_turn == g.st.turn {
                crate::bail!("STADIUM_ALREADY_USED");
            }
            let stadium = match g.st.stadium_card() {
                Some(s) => s,
                None => crate::bail!("NO_STADIUM_IN_PLAY"),
            };
            g.run_fx(Effect::UseStadium { p: p as u8, stadium })?;
        }
        Action::PlayCard { .. } => {}
    }
    Ok(())
}
