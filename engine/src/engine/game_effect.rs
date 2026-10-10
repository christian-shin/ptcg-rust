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

/// `PokemonCardList.clearEffects()` for the modeled fields, except the Special Conditions: a Pokémon that stays in
/// play recovers from them through RemoveCondition events (`engine::condition::recover_by_rule`, called first by
/// every caller), and one leaving play loses them with it (`complete_knock_out`).
pub fn clear_effects(slot: &mut Slot) {
    slot.marker.remove_all_except_trainer_effects();
    slot.poison_damage = 10;
    slot.burn_damage = 20;
    slot.confusion_damage = 30;
    slot.damage_reduction_next_turn = 0;
    slot.healed_this_turn = false;
    slot.cannot_attack_next_turn = false;
    slot.cannot_attack_next_turn_pending = false;
    slot.cannot_retreat_next_turn = false;
    slot.cannot_retreat_next_turn_pending = false;
    slot.cannot_use_attacks_next_turn.clear();
    slot.cannot_use_attacks_next_turn_pending.clear();
    slot.attack_damage_reduction_next_turn = 0;
    slot.blocked_attack_name_next_turn = None;
    slot.blocked_attack_name_until_leaves_active = None;
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
    slot.cannot_attack_next_turn = false;
    slot.cannot_attack_next_turn_pending = false;
    slot.cannot_retreat_next_turn = false;
    slot.cannot_retreat_next_turn_pending = false;
    slot.cannot_use_attacks_next_turn.clear();
    slot.cannot_use_attacks_next_turn_pending.clear();
    slot.attack_damage_reduction_next_turn = 0;
    slot.blocked_attack_name_next_turn = None;
    slot.blocked_attack_name_until_leaves_active = None;
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

/// Put the cards just moved at the top or the bottom of the destination. They were pushed onto
/// its end, so lift them out first (Twinleaf duplicated them and dropped the destination's
/// first cards; fixed in W1-D).
fn reorder_after(g: &mut Game, dst: ListRef, cards: &[CardId], to_top: bool, to_bottom: bool) {
    if !to_top && !to_bottom {
        return;
    }
    let cur: Vec<CardId> = g.lst(dst).to_vec();
    let inside: Vec<CardId> = cards.iter().copied().filter(|c| cur.contains(c)).collect();
    let others: Vec<CardId> = cur.iter().copied().filter(|c| !inside.contains(c)).collect();
    let v: Vec<CardId> = if to_bottom {
        others.iter().chain(inside.iter()).copied().collect()
    } else {
        inside.iter().chain(others.iter()).copied().collect()
    };
    g.lst_mut(dst).set_from(&v);
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
        Effect::MoveCards { .. } => move_cards(g, id),
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

/// Little Grudge `discardSelected(cards)`: a DiscardCardsEffect from a fresh
/// AttackEffect of the grudge owner (source = the slot holding the source
/// card, else the owner's Active) on the prize taker's Active.
pub fn little_grudge_discard(g: &mut Game, owner: usize, prize_taker: usize, attack: AttackRef, source_card: CardId, target: SlotRef, cards: &[CardId]) -> R {
    if cards.is_empty() {
        return Ok(());
    }
    let mut source = SlotRef::new(owner, g.st.players[owner].active);
    for s in g.st.players[owner].in_play().iter() {
        if g.st.slot_pokemon(owner, *s) == Some(source_card) {
            source = SlotRef::new(owner, *s);
        }
    }
    let damage = g.st.cdef(attack.card).attacks[attack.idx()].damage;
    let atk = g.new_fx(Effect::Attack {
        p: owner as u8,
        opp: prize_taker as u8,
        attack,
        damage,
        ignore_weakness: false,
        ignore_resistance: false,
        ignore_defender_effects: false,
        source,
        barrage_used: false,
    });
    // An effect of the Knocked Out Pokémon's own earlier attack (Little Grudge).
    let cause = crate::cause::Cause::attack(owner as u8, Some(source_card), attack);
    let b = AtkBase { attack_effect: atk, player: owner as u8, opponent: prize_taker as u8, attack, source, target, cause };
    let mut cs = SVec::new();
    for c in cards {
        cs.push(*c);
    }
    let r = g.run_fx(Effect::DiscardCards { b, cards: cs });
    g.release_fx(atk);
    r.map(|_| ())
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
