//! `game-effect.ts` (gameReducer) plus ability-lock activation stamps.

use crate::effects::*;
use crate::engine::attack;
use crate::game::{CoinCb, Cont, Game, R};
use crate::list::*;
use crate::markers::*;
use crate::prompts::PromptKind;
use crate::state::*;
use crate::types::*;

pub fn stamp_ability_lock_activation(g: &mut Game, p: usize, slot: SlotId, card: CardId) {
    if !g.st.cdef(card).powers.iter().any(|pw| pw.ability_lock) {
        return;
    }
    g.st.ability_lock_order_counter += 1;
    g.st.players[p].slots[slot as usize].ability_lock_activation_order = g.st.ability_lock_order_counter;
}

pub fn clear_ability_lock_activation(g: &mut Game, card: CardId) {
    if let Some(ListRef::Slot(p, s)) = g.st.locate(card) {
        g.st.players[p as usize].slots[s as usize].ability_lock_activation_order = 0;
    }
}

pub fn stamp_starting_ability_locks(g: &mut Game) {
    g.st.ability_lock_order_counter += 1;
    let order = g.st.ability_lock_order_counter;
    for p in 0..2 {
        if let Some(c) = g.st.active_pokemon(p) {
            if g.st.cdef(c).powers.iter().any(|pw| pw.ability_lock) {
                let a = g.st.players[p].active;
                g.st.players[p].slots[a as usize].ability_lock_activation_order = order;
            }
        }
    }
}

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
    slot.is_public = is_public;
    // attacksThisTurn = undefined
    slot.attacks_this_turn = None;
}

/// `PokemonCardList.clearEffects()` for the modeled fields.
pub fn clear_effects(slot: &mut Slot) {
    slot.marker.clear();
    for sc in [SpecialCondition::Poisoned, SpecialCondition::Asleep, SpecialCondition::Burned, SpecialCondition::Confused, SpecialCondition::Paralyzed] {
        let v = sc as u8;
        slot.special_conditions.retain(|x| *x != v);
    }
    slot.poison_damage = 10;
    slot.burn_damage = 20;
    slot.confusion_damage = 30;
    slot.damage_reduction_next_turn = 0;
    slot.prevent_damage_next_turn = false;
    slot.prevent_damage_next_turn_pending = false;
    slot.cannot_be_healed_next_turn = false;
    slot.healed_this_turn = false;
    slot.cannot_attack_next_turn = false;
    slot.cannot_attack_next_turn_pending = false;
    slot.cannot_retreat_next_turn = false;
    slot.cannot_retreat_next_turn_pending = false;
    slot.cannot_use_attacks_next_turn.clear();
    slot.cannot_use_attacks_next_turn_pending.clear();
    slot.attack_damage_reduction_next_turn = 0;
    slot.blocked_attack_name_next_turn = None;
}

/// `PokemonCardList.removeAttackEffects()` for the modeled fields.
pub fn remove_attack_effects(slot: &mut Slot) {
    slot.marker.remove_attack_effects();
    slot.cannot_attack_next_turn = false;
    slot.cannot_attack_next_turn_pending = false;
    slot.cannot_retreat_next_turn = false;
    slot.cannot_retreat_next_turn_pending = false;
    slot.cannot_use_attacks_next_turn.clear();
    slot.cannot_use_attacks_next_turn_pending.clear();
    slot.attack_damage_reduction_next_turn = 0;
    slot.blocked_attack_name_next_turn = None;
    slot.damage_reduction_next_turn = 0;
    slot.prevent_damage_next_turn = false;
    slot.prevent_damage_next_turn_pending = false;
    slot.cannot_be_healed_next_turn = false;
    slot.healed_this_turn = false;
}

fn is_discard_pile(r: ListRef) -> bool {
    matches!(r, ListRef::Discard(_))
}

fn prism_split(g: &Game, cards: &[CardId]) -> (Vec<CardId>, Vec<CardId>) {
    let mut lost = Vec::new();
    let mut disc = Vec::new();
    for &c in cards {
        if g.st.cdef(c).has_tag(tag::PRISM_STAR) {
            lost.push(c);
        } else {
            disc.push(c);
        }
    }
    (lost, disc)
}

fn move_with_prism_check(g: &mut Game, cards: &[CardId], src: ListRef, dst: ListRef) {
    if is_discard_pile(dst) {
        let owner = dst.owner().unwrap() as u8;
        let (lost, disc) = prism_split(g, cards);
        if !lost.is_empty() {
            g.move_cards_to(src, &lost, ListRef::LostZone(owner));
        }
        if !disc.is_empty() {
            g.move_cards_to(src, &disc, dst);
        }
    } else {
        g.move_cards_to(src, cards, dst);
    }
}

fn reorder_after(g: &mut Game, dst: ListRef, cards: &[CardId], to_top: bool, to_bottom: bool) {
    if to_bottom {
        let cur: Vec<CardId> = g.lst(dst).to_vec();
        let mut v: Vec<CardId> = cur.iter().skip(cards.len()).copied().collect();
        v.extend_from_slice(cards);
        g.lst_mut(dst).set_from(&v);
    } else if to_top {
        let mut v: Vec<CardId> = cards.to_vec();
        v.extend_from_slice(g.lst(dst));
        g.lst_mut(dst).set_from(&v);
    }
}

fn move_cards(g: &mut Game, id: EffId) -> R {
    let (source, destination, cards, count, to_top, to_bottom, skip_cleanup) = match *g.e(id) {
        Effect::MoveCards { source, destination, cards, count, to_top, to_bottom, skip_cleanup, .. } => {
            (source, destination, cards, count, to_top, to_bottom, skip_cleanup)
        }
        _ => return Ok(()),
    };
    let partial = cards.is_some() || count.is_some();
    if let ListRef::Slot(p, s) = source {
        if !skip_cleanup && !partial {
            let tools: Vec<CardId> = g.st.players[p as usize].slots[s as usize].tools.iter().collect();
            for t in tools {
                g.move_card_to(source, t, destination);
            }
        }
    }
    if let Some(cs) = cards {
        move_with_prism_check(g, cs.as_slice(), source, destination);
        reorder_after(g, destination, cs.as_slice(), to_top, to_bottom);
    } else if let Some(n) = count {
        let cs: Vec<CardId> = g.lst(source).iter().take(n.max(0) as usize).copied().collect();
        move_with_prism_check(g, &cs, source, destination);
        reorder_after(g, destination, &cs, to_top, to_bottom);
    } else if is_discard_pile(destination) {
        let owner = destination.owner().unwrap() as u8;
        let all: Vec<CardId> = g.lst(source).to_vec();
        let (lost, disc) = prism_split(g, &all);
        if !lost.is_empty() {
            g.move_cards_to(source, &lost, ListRef::LostZone(owner));
        }
        if !disc.is_empty() {
            g.move_cards_to(source, &disc, destination);
        }
    } else if to_top {
        g.move_to_top_of_destination(source, destination);
    } else {
        g.move_to(source, destination, None);
    }

    if let ListRef::Slot(p, s) = source {
        let (pu, su) = (p as usize, s);
        if g.st.slot_pokemons(pu, su).is_empty() {
            let rest: Vec<CardId> = g.st.slot(pu, su).cards.iter().collect();
            if !rest.is_empty() {
                let (lost, disc) = prism_split(g, &rest);
                if !lost.is_empty() {
                    g.move_cards_to(source, &lost, ListRef::LostZone(p));
                }
                if !disc.is_empty() {
                    g.move_cards_to(source, &disc, ListRef::Discard(p));
                }
            }
            g.st.players[pu].slots[su as usize].energies.clear();
            let tools: Vec<CardId> = g.st.slot(pu, su).tools.iter().collect();
            for t in tools {
                let dst = if g.st.cdef(t).has_tag(tag::PRISM_STAR) { ListRef::LostZone(p) } else { ListRef::Discard(p) };
                g.move_card_to(source, t, dst);
            }
        }
        if !skip_cleanup && g.st.slot_pokemons(pu, su).is_empty() {
            reset_empty_slot(&mut g.st.players[pu].slots[su as usize]);
        }
    }
    Ok(())
}

fn knock_out(g: &mut Game, id: EffId) -> R {
    let (p, target) = match *g.e(id) {
        Effect::KnockOut { p, target, .. } => (p as usize, target),
        _ => return Ok(()),
    };
    let card = match g.st.slot_pokemon(target.p as usize, target.s) {
        Some(c) => c,
        None => return Ok(()),
    };
    let d = g.st.cdef(card);
    let mut extra = 0;
    if d.has_tag(tag::POKEMON_EX) || d.has_tag(tag::POKEMON_V) || d.has_tag(tag::POKEMON_VSTAR) || d.has_tag(tag::POKEMON_EX_LOWER) || d.has_tag(tag::POKEMON_GX) {
        extra += 1;
    }
    if d.has_tag(tag::POKEMON_SV_MEGA) || d.has_tag(tag::TAG_TEAM) || d.has_tag(tag::DUAL_LEGEND) {
        extra += 1;
    }
    if d.has_tag(tag::POKEMON_VMAX) || d.has_tag(tag::POKEMON_VUNION) {
        extra += 2;
    }
    if let Effect::KnockOut { prize_count, .. } = g.e_mut(id) {
        *prize_count += extra;
    }
    // Prize denial / extra prizes / Little Grudge: not modeled.

    let owner = p;
    let attacker = 1 - p;
    let during_opp_turn = matches!(g.st.phase, GamePhase::PlayerTurn | GamePhase::Attack) && g.st.active_player as usize == attacker;
    if during_opp_turn {
        g.st.players[owner].pokemon_knocked_out_during_opponents_last_turn = true;
        let def_id = g.st.cards[card as usize].def;
        g.st.players[owner].pokemon_knocked_out_last_turn_entries.push(def_id);
    }
    if g.st.phase == GamePhase::Attack
        && g.st.active_player as usize == attacker
        && g.st.players[owner].marker.has(DAMAGE_DEALT_MARKER)
    {
        g.st.players[owner].pokemon_knocked_out_by_attack_during_opponents_last_turn = true;
    }
    let tp = target.p as usize;
    if g.st.slot(tp, target.s).marker.has(LOST_CITY_MARKER) || d.has_tag(tag::PRISM_STAR) {
        crate::bail!("LOST_CITY_KO_NOT_PORTED");
    }
    let tools: Vec<CardId> = g.st.slot(tp, target.s).tools.iter().collect();
    for t in tools {
        g.move_card_to(target.list(), t, ListRef::Discard(owner as u8));
    }
    clear_effects(&mut g.st.players[tp].slots[target.s as usize]);
    g.run_fx(Effect::MoveCards {
        source: target.list(),
        destination: ListRef::Discard(owner as u8),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: NO_CARD,
    })?;
    Ok(())
}

pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::KnockOut { .. } => knock_out(g, id),
        Effect::CheckPokemonStats { .. } => {
            // noWeaknessNextTurn / weaknessOverride / opponent weakness aura: not modeled.
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
        Effect::Heal { target, damage, .. } => {
            let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
            if slot.cannot_be_healed_next_turn {
                g.set_prevent(id, true);
                return Ok(());
            }
            if damage > 0 && slot.damage > 0 {
                slot.healed_this_turn = true;
            }
            slot.damage = (slot.damage - damage).max(0);
            Ok(())
        }
        Effect::PlaceDamageCounters { target, damage, .. } => {
            if g.st.slot_pokemon(target.p as usize, target.s).is_none() {
                crate::bail!("ILLEGAL_ACTION");
            }
            g.st.players[target.p as usize].slots[target.s as usize].damage += damage.max(0);
            Ok(())
        }
        Effect::Evolve { p, target, card } => evolve(g, p as usize, target, card),
        Effect::AddSpecialConditionsPower { target, conditions, poison_damage, burn_damage, sleep_flips, confusion_damage, .. } => {
            let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
            for &c in conditions.iter() {
                crate::engine::phase::add_condition(slot, SpecialCondition::from_u8(c));
            }
            slot.poison_damage = poison_damage;
            slot.burn_damage = burn_damage;
            slot.confusion_damage = confusion_damage;
            slot.sleep_flips = sleep_flips;
            Ok(())
        }
        Effect::MoveCards { .. } => move_cards(g, id),
        Effect::CoinFlipSequence { p, mode, callback, .. } => {
            let cb = CoinCb::Sequence { p, mode, results: 0, n: 0, callback };
            g.coin_callbacks.push(cb);
            let k = (g.coin_callbacks.len() - 1) as u8;
            g.run_fx(Effect::CoinFlip { p, callback: Some(k), result: None, skip_reflip_stadium: true, skip_reflip_tool: true })?;
            Ok(())
        }
        Effect::CoinFlip { p, callback, .. } => {
            let result = g.rng.coin();
            if let Effect::CoinFlip { result: r, .. } = g.e_mut(id) {
                *r = Some(result);
            }
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

fn evolve(g: &mut Game, p: usize, target: SlotRef, card: CardId) -> R {
    let pl = &g.st.players[p];
    if pl.cannot_play_pokemon_cards || pl.cannot_evolve_pokemon_cards {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    if pl.cannot_play_pokemon_with_abilities && g.st.cdef(card).powers.iter().any(|pw| pw.power_type == PowerType::Ability as u8) {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    if g.st.slot_pokemon(target.p as usize, target.s).is_none() {
        crate::bail!("INVALID_TARGET");
    }
    g.move_card_to(ListRef::Hand(p as u8), card, target.list());
    let turn = g.st.turn;
    let slot = &mut g.st.players[target.p as usize].slots[target.s as usize];
    slot.pokemon_played_turn = turn;
    slot.marker.clear();
    if g.st.players[p].active == target.s && target.p as usize == p {
        g.st.players[p].slots[target.s as usize].ability_lock_activation_order = 0;
        stamp_ability_lock_activation(g, p, target.s, card);
    }
    Ok(())
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
