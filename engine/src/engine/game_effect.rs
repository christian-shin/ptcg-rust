//! The game-level reducer: effects on the game itself and on a slot's records.

use crate::effects::*;
use crate::engine::attack;
use crate::game::{Game, R};
use crate::list::*;
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

/// Spot `s` of player `p` keeps its cards and whether it is public; everything else on it,
/// its lasting effects included, ends.
pub fn reset_empty_slot(st: &mut State, p: usize, s: SlotId) {
    st.slot_edit(p, s).reset_keeping_cards();
    crate::engine::lasting::clear_spot(st, p, s);
}

/// What evolving or devolving does to the effects on the Pokémon: every effect on it ends, a Trainer's included
/// (APR C-13: "Any Special Conditions or effects it had on it are removed"; A-05; official JP FAQ: Acerola's Mischief on
/// Mega Gardevoir ex devolved by Strange Timepiece: 「はい、なくなります。」; Greninja's Ability: 「特性による「持続する
/// 効果」であっても、ポケモンが進化・退化・レベルアップすることで、なくなります。」 = even an Ability's lasting effect ends
/// when the Pokémon evolves, devolves or levels up). Moving to the Bench keeps a Trainer's or an Ability's effect
/// ([`clear_effects`]; id2228, id1651).
pub fn clear_effects_evolving(st: &mut State, p: usize, s: SlotId) {
    clear_effects(st, p, s);
    crate::engine::lasting::clear_declared(st, p, s);
}

/// Clears the effects on a Pokémon for the modeled fields, except the Special Conditions: a Pokémon that stays in
/// play recovers from them through RemoveCondition events (`engine::condition::recover_by_rule`, called first by
/// every caller), and one leaving play loses them with it (the LeavePlay reducer, `engine::knockout`). The lasting
/// effects on it end but the cost increases and the next-turn bonus (`lasting::clear_effects`).
pub fn clear_effects(st: &mut State, p: usize, s: SlotId) {
    let slot = &mut st.players[p].slots[s as usize];
    slot.poison_damage = 10;
    slot.burn_damage = 20;
    slot.confusion_damage = 30;
    slot.healed_this_turn = false;
    crate::engine::lasting::clear_effects(st, p, s);
}

/// Ends the effects of attacks on a Pokémon for the modeled fields: every lasting effect on the Pokémon ends.
pub fn remove_attack_effects(st: &mut State, p: usize, s: SlotId) {
    let slot = &mut st.players[p].slots[s as usize];
    slot.healed_this_turn = false;
    crate::engine::lasting::remove_attack_effects(st, p, s);
}

/// Steps 3-4 of the damage calculation (APR B-06, B-07): the damage after the Weakness and Resistance of the Pokémon in
/// `b.target` against the types of the attacking Pokémon (`ignore_*`: the attack's own text ignores them).
pub fn weakness_and_resistance(g: &mut Game, b: &AtkBase, damage: i32, ignore_weakness: bool, ignore_resistance: bool) -> R<i32> {
    let types = crate::derived::types(g, b.source)?;
    let (w, r) = crate::derived::weakness_resistance(g, b.target)?;
    let weak: &[WeaknessV] = if ignore_weakness { &[] } else { w.as_slice() };
    let res: &[ResistanceV] = if ignore_resistance { &[] } else { r.as_slice() };
    Ok(apply_weakness_and_resistance(damage, types.as_slice(), weak, res))
}

pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::UseAttack { .. } => attack::start_use_attack(g, id),
        Effect::UsePower { .. } => attack::start_use_power(g, id),
        Effect::UseStadium { p, .. } => {
            g.st.players[p as usize].stadium_used_turn = g.st.turn;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// The printed types the derived types start from, for a slot.
pub fn pokemon_types(g: &Game, s: SlotRef) -> SVec<CardType, 4> {
    let mut v = SVec::new();
    if let Some(c) = g.st.slot_pokemon(s.p as usize, s.s) {
        for &t in g.st.cdef(c).card_type {
            v.push(t);
        }
    }
    v
}

/// The printed Weakness and Resistance of the Pokémon in a slot.
pub fn printed_stats(g: &Game, s: SlotRef) -> (SVec<WeaknessV, 4>, SVec<ResistanceV, 4>) {
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
    (weakness, resistance)
}

/// A lasting "when this Pokémon is Knocked Out" trigger that discards Energy from the Attacking Pokémon: the chosen Energy of
/// the Attacking Pokémon (`target`, the Prize taker's) leave play for its owner's discard pile, an effect of the Knocked Out
/// Pokémon's own earlier attack (the LeavePlay of attached cards, which the attack-effect preventions on that Pokémon stop:
/// Mist Energy).
pub fn discard_attacker_energy(g: &mut Game, owner: usize, attack: AttackRef, source_card: CardId, target: SlotRef, cards: &[CardId]) -> R {
    if cards.is_empty() {
        return Ok(());
    }
    let cause = crate::cause::Cause::attack(owner as u8, Some(source_card), attack);
    crate::engine::knockout::leave_play_cards(g, target, cards, crate::spec::event::RulesZone::Discard, cause, None)?;
    Ok(())
}

/// The answer of the prompt that asks which Energy of the Attacking Pokémon to discard ([`discard_attacker_energy`]).
pub fn discard_attacker_energy_cont(g: &mut Game, owner: u8, attack: AttackRef, source_card: CardId, target: SlotRef, res: crate::prompts::Res) -> R {
    let cards: Vec<CardId> = match res {
        crate::prompts::Res::CardsFrom(t) => t.iter().map(|x| x.1).collect(),
        _ => Vec::new(),
    };
    if cards.is_empty() {
        return Ok(());
    }
    discard_attacker_energy(g, owner as usize, attack, source_card, target, &cards)
}
