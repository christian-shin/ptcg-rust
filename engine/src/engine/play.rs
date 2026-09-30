//! `play-energy-effect.ts`, `play-pokemon-effect.ts`, `play-trainer-effect.ts`.

use crate::effects::*;
use crate::engine::game_effect::clear_effects;
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::markers::*;
use crate::state::*;
use crate::types::*;

pub fn play_energy_reducer(g: &mut Game, id: EffId) -> R {
    let (p, card, target) = match *g.e(id) {
        Effect::AttachEnergy { p, card, target } => (p as usize, card, target),
        _ => return Ok(()),
    };
    if g.st.slot_pokemon(target.p as usize, target.s).is_none() {
        crate::bail!("INVALID_TARGET");
    }
    let pl = &g.st.players[p];
    if pl.cannot_play_energy_cards {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    if g.st.cdef(card).energy_type == EnergyType::Special as u8 && pl.cannot_play_special_energy_cards {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    // cannotAttachEnergyFromHandNextTurn / pending attach consequences: not modeled.
    g.move_card_to(ListRef::Hand(p as u8), card, target.list());
    let e = &mut g.st.players[target.p as usize].slots[target.s as usize].energies;
    if !e.contains(card) {
        e.push(card);
    }
    Ok(())
}

fn can_evolve_from(g: &Game, base: CardId, evo: CardId) -> bool {
    let b = g.st.cdef(base);
    let e = g.st.cdef(evo);
    (b.stage < e.stage && b.name == e.evolves_from)
        || b.evolves_to.contains(&e.name)
        || b.evolves_to_stage.contains(&e.stage)
        || (!b.evolves_from_base.is_empty() && b.evolves_from_base.contains(&e.evolves_from))
}

/// `play-pokemon-from-deck-effect.ts` / `...-from-discard-effect.ts`.
pub fn play_pokemon_from_zone_reducer(g: &mut Game, id: EffId) -> R {
    let (p, card, target, src) = match *g.e(id) {
        Effect::PlayPokemonFromDeck { p, card, target } => (p, card, target, ListRef::Deck(p)),
        Effect::PlayPokemonFromDiscard { p, card, target } => (p, card, target, ListRef::Discard(p)),
        _ => return Ok(()),
    };
    if g.st.slot(target.p as usize, target.s).cards.is_empty() {
        g.move_card_to(src, card, target.list());
        let turn = g.st.turn;
        g.st.players[target.p as usize].slots[target.s as usize].pokemon_played_turn = turn;
        let _ = p;
        return Ok(());
    }
    crate::bail!("INVALID_TARGET");
}

pub fn play_pokemon_reducer(g: &mut Game, id: EffId) -> R {
    let (p, card, target) = match *g.e(id) {
        Effect::PlayPokemon { p, card, target, .. } => (p as usize, card, target),
        _ => return Ok(()),
    };
    let d = g.st.cdef(card);
    let pl = &g.st.players[p];
    if pl.cannot_play_pokemon_cards {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    if pl.cannot_play_pokemon_with_abilities && d.powers.iter().any(|pw| pw.power_type == PowerType::Ability as u8) {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    let tslot = g.st.slot(target.p as usize, target.s);
    if d.stage == Stage::Basic as u8 && tslot.cards.is_empty() {
        g.move_card_to(ListRef::Hand(p as u8), card, target.list());
        let turn = g.st.turn;
        g.st.players[target.p as usize].slots[target.s as usize].pokemon_played_turn = turn;
        return Ok(());
    }
    let base = match g.st.slot_pokemon(target.p as usize, target.s) {
        Some(c) => c,
        None => crate::bail!("INVALID_TARGET"),
    };
    if !can_evolve_from(g, base, card) {
        crate::bail!("INVALID_TARGET");
    }
    if g.st.players[p].cannot_evolve_pokemon_cards {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    let played = g.st.slot(target.p as usize, target.s).pokemon_played_turn;
    let (e, _) = g.run_fx(Effect::CheckPokemonPlayedTurn { p: p as u8, target, pokemon_played_turn: played })?;
    let played = match e {
        Effect::CheckPokemonPlayedTurn { pokemon_played_turn, .. } => pokemon_played_turn,
        _ => played,
    };
    let turn = g.st.turn;
    if (turn == 0 || turn == 1 || turn == 2) && !g.st.players[p].can_evolve {
        crate::bail!("CANNOT_EVOLVE_ON_YOUR_FIRST_TURN");
    }
    if played >= turn {
        crate::bail!("POKEMON_CANT_EVOLVE_THIS_TURN");
    }
    g.run_fx(Effect::Evolve { p: p as u8, target, card })?;
    let (e, _) = g.run_fx(Effect::CheckSpecialConditionRemoval { p: p as u8, target, preserved: SVec::new() })?;
    let preserved = match e {
        Effect::CheckSpecialConditionRemoval { preserved, .. } => preserved,
        _ => SVec::new(),
    };
    // player.removePokemonEffects(target)
    g.st.players[p].marker.remove(KNOCKOUT_MARKER);
    g.st.players[p].marker.remove(CLEAR_KNOCKOUT_MARKER);
    let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
    let keep: SVec<u8, 5> = {
        let mut v = SVec::new();
        for c in slot.special_conditions.iter() {
            if preserved.contains(c) {
                v.push(*c);
            }
        }
        v
    };
    let before = slot.special_conditions;
    clear_effects(slot);
    // clearEffects only removes non-preserved conditions (order kept).
    slot.special_conditions = before;
    slot.special_conditions.retain(|c| keep.contains(c));
    slot.marker.clear();
    slot.board_effect.retain(|b| *b != BoardEffect::AbilityUsed as u8);
    Ok(())
}

fn cleanup_target(g: &Game, card: CardId) -> fn(u8) -> ListRef {
    if g.st.cdef(card).has_tag(tag::PRISM_STAR) {
        ListRef::LostZone
    } else {
        ListRef::Discard
    }
}

pub fn trainer_cleanup(g: &mut Game, p: usize, card: CardId) {
    if !g.st.players[p].supporter.contains(card) {
        return;
    }
    let t = cleanup_target(g, card)(p as u8);
    g.move_card_to(ListRef::Supporter(p as u8), card, t);
}

fn restore_played_trainer(g: &mut Game, p: usize, card: CardId) {
    if !g.has_prompts() {
        return;
    }
    if g.st.players[p].discard.contains(card) {
        g.move_card_to(ListRef::Discard(p as u8), card, ListRef::Supporter(p as u8));
    }
}

fn finalize_trainer_cleanup(g: &mut Game, p: usize, card: CardId, keep: bool) {
    if keep {
        return;
    }
    if g.has_prompts() {
        g.wait_prompt(Cont::TrainerCleanup { p: p as u8, card });
        return;
    }
    trainer_cleanup(g, p, card);
}

pub fn play_trainer_reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::PlaySupporter { p, card, target } => {
            let pu = p as usize;
            if g.st.players[pu].cannot_play_supporter_cards {
                crate::bail!("BLOCKED_BY_EFFECT");
            }
            // coinFlipCancelTrainerPlay: not modeled (flag never set by pool cards).
            g.run_fx(Effect::Trainer { p, card, target })?;
            restore_played_trainer(g, pu, card);
            let keep = g.st.rules.supporter_cleanup_at_end_turn;
            finalize_trainer_cleanup(g, pu, card, keep);
            g.st.players[pu].supporter_turn += 1;
            Ok(())
        }
        Effect::PlayStadium { p, card } => {
            let pu = p as usize;
            let stadium = g.st.stadium_card();
            if g.st.players[pu].cannot_play_stadium_cards {
                crate::bail!("BLOCKED_BY_EFFECT");
            }
            let prism = stadium.map(|s| g.st.cdef(s).has_tag(tag::PRISM_STAR)).unwrap_or(false);
            for q in [pu, 1 - pu] {
                if !g.st.players[q].stadium.is_empty() {
                    let dst = if prism { ListRef::LostZone(q as u8) } else { ListRef::Discard(q as u8) };
                    g.move_to(ListRef::Stadium(q as u8), dst, None);
                }
            }
            g.st.players[pu].stadium_used_turn = 0;
            g.move_card_to(ListRef::Hand(p), card, ListRef::Stadium(p));
            Ok(())
        }
        Effect::AttachPokemonTool { p, card, target } => {
            let pu = p as usize;
            if target.p != p && !g.st.cdef(card).attaches_to_opponents_pokemon {
                crate::bail!("INVALID_TARGET");
            }
            let pc = match g.st.slot_pokemon(target.p as usize, target.s) {
                Some(c) => c,
                None => crate::bail!("INVALID_TARGET"),
            };
            if g.st.slot(target.p as usize, target.s).tools.len() >= g.st.cdef(pc).max_tools as usize {
                crate::bail!("POKEMON_TOOL_ALREADY_ATTACHED");
            }
            if g.st.players[pu].cannot_play_tool_cards {
                crate::bail!("BLOCKED_BY_EFFECT");
            }
            g.move_card_to(ListRef::Hand(p), card, target.list());
            let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
            slot.cards.remove(card);
            slot.tools.push(card);
            g.run_fx(Effect::Trainer { p, card, target: Some(target) })?;
            Ok(())
        }
        Effect::PlayItem { p, card, target } => {
            let pu = p as usize;
            if g.st.players[pu].cannot_play_item_cards {
                crate::bail!("BLOCKED_BY_EFFECT");
            }
            g.move_card_to(ListRef::Hand(p), card, ListRef::Supporter(p));
            g.run_fx(Effect::Trainer { p, card, target })?;
            restore_played_trainer(g, pu, card);
            finalize_trainer_cleanup(g, pu, card, false);
            Ok(())
        }
        Effect::Trainer { p, card, .. } => {
            if g.st.players[p as usize].hand.contains(card) {
                let dst = if g.st.cdef(card).trainer_type == TrainerType::Supporter as u8 {
                    ListRef::Supporter(p)
                } else {
                    ListRef::Discard(p)
                };
                g.move_card_to(ListRef::Hand(p), card, dst);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
