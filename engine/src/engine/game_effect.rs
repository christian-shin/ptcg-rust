//! `game-effect.ts` (gameReducer).

use crate::effects::*;
use crate::engine::attack;
use crate::game::{CoinCb, Cont, Game, R};
use crate::list::*;
use crate::prompts::PromptKind;
use crate::state::*;
use crate::types::*;

pub fn apply_weakness_and_resistance(damage: i32, types: &[CardType], weakness: &[WeaknessV], resistance: &[ResistanceV]) -> i32 {
    let mut multiply = 1;
    let mut modifier = 0;
    for w in weakness {
        if types.contains(&w.card_type) {
            match w.value {
                None => multiply *= 2,
                Some(v) => modifier += v,
            }
        }
    }
    for r in resistance {
        if types.contains(&r.card_type) {
            modifier += r.value;
        }
    }
    damage * multiply + modifier
}

/// `resetEmptyPokemonSlot`.
pub fn reset_empty_slot(slot: &mut Slot) {
    let is_public = slot.is_public;
    let cards = slot.cards;
    let energies = slot.energies;
    *slot = Slot::default();
    slot.cards = cards;
    slot.energies = energies;
    touch();
    slot.is_public = is_public;
    // attacksThisTurn = undefined
    slot.attacks_this_turn = None;
}

/// What evolving or devolving does to the effects on the Pokémon: every effect on it ends, a Trainer's included
/// (APR C-13: "Any Special Conditions or effects it had on it are removed"; A-05; official JP FAQ: Acerola's Mischief on
/// Mega Gardevoir ex devolved by Strange Timepiece: 「はい、なくなります。」; Greninja's Ability: 「特性による「持続する
/// 効果」であっても、ポケモンが進化・退化・レベルアップすることで、なくなります。」 = even an Ability's lasting effect ends
/// when the Pokémon evolves, devolves or levels up). Moving to the Bench keeps a Trainer's or an Ability's effect
/// ([`clear_effects`]; id2228, id1651).
pub fn clear_effects_evolving(slot: &mut Slot) {
    clear_effects(slot);
    slot.marker.clear();
}

/// `PokemonCardList.clearEffects()` for the modeled fields, except the Special Conditions: a Pokémon that stays in
/// play recovers from them through RemoveCondition events (`engine::condition::recover_by_rule`, called first by
/// every caller), and one leaving play loses them with it (the LeavePlay reducer, `engine::knockout`).
pub fn clear_effects(slot: &mut Slot) {
    slot.marker.remove_all_except_trainer_effects();
    slot.poison_damage = 10;
    slot.burn_damage = 20;
    slot.confusion_damage = 30;
    slot.damage_reduction_next_turn = 0;
    slot.healed_this_turn = false;
    slot.lasting_locks.clear();
    slot.attack_damage_reduction_next_turn = 0;
    clear_prevent_next_turn(slot);
}

fn clear_prevent_next_turn(slot: &mut Slot) {
    slot.no_weakness_next_turn = false;
    slot.no_weakness_next_turn_pending = false;
    slot.discard_attacker_energy_if_ko_next_turn = false;
    slot.discard_attacker_energy_if_ko_next_turn_pending = false;
    slot.discard_attacker_energy_if_ko_attack = None;
    slot.discard_attacker_energy_if_ko_source_card = None;
    slot.discard_attacker_energy_if_ko_attacker = None;
    slot.defending_extra_damage_next_turn = 0;
    slot.defending_extra_damage_attacker = None;
    slot.defending_extra_damage_pending = false;
    slot.defending_extra_damage_rearm_after_attack = false;
    slot.retaliate_on_damage_next_turn = None;
    slot.retaliate_on_damage_next_turn_pending = None;
    slot.lasting_prevents.clear();
}

/// `PokemonCardList.removeAttackEffects()` for the modeled fields.
pub fn remove_attack_effects(slot: &mut Slot) {
    slot.marker.remove_attack_effects();
    slot.lasting_locks.clear();
    slot.attack_damage_reduction_next_turn = 0;
    slot.damage_reduction_next_turn = 0;
    slot.healed_this_turn = false;
    slot.attack_cost_increase_next_turn = 0;
    slot.attack_cost_increase_next_turn_pending = 0;
    slot.attack_cost_increase_next_turn_attacker = None;
    slot.retreat_cost_increase_next_turn = 0;
    slot.retreat_cost_increase_next_turn_pending = 0;
    slot.retreat_cost_increase_next_turn_attacker = None;
    clear_prevent_next_turn(slot);
    slot.next_turn_attack_damage_bonus = None;
    slot.next_turn_attack_damage_bonus_pending = None;
}

pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::CheckPokemonStats { target, .. } => {
            if g.st.slot(target.p as usize, target.s).no_weakness_next_turn {
                if let Effect::CheckPokemonStats { weakness, .. } = g.e_mut(id) {
                    weakness.clear();
                }
            }
            // weaknessOverride / opponent weakness aura: not modeled.
            Ok(())
        }
        Effect::ApplyWeakness { b, damage, ignore_weakness, ignore_resistance } => {
            let (t, _) = g.run_fx(Effect::CheckPokemonType { target: b.source, card_types: pokemon_types(g, b.source) })?;
            let (s, _) = g.run_fx(stats_effect(g, b.target))?;
            let types = match t {
                Effect::CheckPokemonType { card_types, .. } => card_types,
                _ => SVec::new(),
            };
            let (w, r) = match s {
                Effect::CheckPokemonStats { weakness, resistance, .. } => (weakness, resistance),
                _ => (SVec::new(), SVec::new()),
            };
            let weak: &[WeaknessV] = if ignore_weakness { &[] } else { w.as_slice() };
            let res: &[ResistanceV] = if ignore_resistance { &[] } else { r.as_slice() };
            let dmg = apply_weakness_and_resistance(damage, types.as_slice(), weak, res);
            if let Effect::ApplyWeakness { damage, .. } = g.e_mut(id) {
                *damage = dmg;
            }
            Ok(())
        }
        Effect::UseAttack { .. } => attack::start_use_attack(g, id),
        Effect::UsePower { .. } => attack::start_use_power(g, id),
        Effect::UseStadium { p, .. } => {
            if g.st.players.iter().any(|pl| pl.stadium_and_tool_have_no_effect_turns_remaining > 0) {
                crate::bail!("BLOCKED_BY_EFFECT");
            }
            g.st.players[p as usize].stadium_used_turn = g.st.turn;
            Ok(())
        }
        Effect::CoinFlipSequence { p, mode, callback, cause, .. } => {
            let cb = CoinCb::Sequence { p, mode, results: 0, n: 0, callback, cause };
            g.coin_callbacks.push(cb);
            let k = (g.coin_callbacks.len() - 1) as u8;
            g.run_fx_unit(Effect::CoinFlipRequest { p, callback: Some(k), result: None, skip_reflip_stadium: true, skip_reflip_tool: true, cause })?;
            Ok(())
        }
        Effect::CoinFlipRequest { p, callback, cause, .. } => {
            let result = g.rng.coin();
            if let Effect::CoinFlipRequest { result: r, .. } = g.e_mut(id) {
                *r = Some(result);
            }
            crate::engine::condition::coin_flipped(g, p as usize, crate::spec::event::CoinPurpose::Effect, result, cause)?;
            let cb = match callback {
                Some(k) => g.coin_callbacks.as_slice()[k as usize],
                None => CoinCb::None,
            };
            let pid = g.player_id(p as usize);
            g.prompt(pid, "", PromptKind::Wait, Cont::CoinFlipWait { cb, result });
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Initial `CheckPokemonTypeEffect.cardTypes` for a slot.
pub fn pokemon_types(g: &Game, s: SlotRef) -> SVec<CardType, 4> {
    let mut v = SVec::new();
    if let Some(c) = g.st.slot_pokemon(s.p as usize, s.s) {
        for &t in g.st.cdef(c).card_type {
            v.push(t);
        }
    }
    v
}

/// A fresh `CheckPokemonStatsEffect` for a slot.
pub fn stats_effect(g: &Game, s: SlotRef) -> Effect {
    let mut weakness = SVec::new();
    let mut resistance = SVec::new();
    if let Some(c) = g.st.slot_pokemon(s.p as usize, s.s) {
        let d = g.st.cdef(c);
        for w in d.weakness {
            weakness.push(WeaknessV { card_type: w.card_type, value: w.value });
        }
        for r in d.resistance {
            resistance.push(ResistanceV { card_type: r.card_type, value: r.value });
        }
    }
    Effect::CheckPokemonStats { target: s, weakness, resistance }
}

/// Little Grudge: the chosen Energy of the Attacking Pokémon (`target`, the Prize taker's) leave play for its owner's
/// discard pile, an effect of the Knocked Out Pokémon's own earlier attack (the LeavePlay of attached cards, which the
/// attack-effect preventions on that Pokémon stop: Mist Energy).
pub fn little_grudge_discard(g: &mut Game, owner: usize, _prize_taker: usize, attack: AttackRef, source_card: CardId, target: SlotRef, cards: &[CardId]) -> R {
    if cards.is_empty() {
        return Ok(());
    }
    let cause = crate::cause::Cause::attack(owner as u8, Some(source_card), attack);
    crate::engine::knockout::leave_play_cards(g, target, cards, crate::spec::event::RulesZone::Discard, cause, None)?;
    Ok(())
}

/// DiscardEnergyPrompt callback of Little Grudge.
pub fn little_grudge_cont(g: &mut Game, owner: u8, prize_taker: u8, attack: AttackRef, source_card: CardId, target: SlotRef, res: crate::prompts::Res) -> R {
    let cards: Vec<CardId> = match res {
        crate::prompts::Res::CardsFrom(t) => t.iter().map(|x| x.1).collect(),
        _ => Vec::new(),
    };
    if cards.is_empty() {
        return Ok(());
    }
    little_grudge_discard(g, owner as usize, prize_taker as usize, attack, source_card, target, &cards)
}
