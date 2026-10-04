//! Antique Root Fossil (SCR): play this card as a 60 HP Basic [C] Pokémon.
//! It can't be affected by Special Conditions and can't retreat. At any time
//! during your turn, you may discard it from play.
//!
//! Primal Root: as long as it is Active, attacks used by your opponent's
//! Basic Pokémon cost [C] more.
//!
//! Fixed (phase 4b, W4): Twinleaf did not implement Primal Root at all
//! (CheckAttackCostEffect of an attacker whose Active is Basic, with this card
//! the opponent's Active top card and the ability not blocked: +[C]). The
//! printed `powers` list still holds only the discard power.
//!
//! Twinleaf: on its own PlayItemEffect the card reduces a PlayPokemonEffect
//! into the first empty Bench slot (the item play then continues and finds
//! the card no longer in hand). In play, its Trainer Ability (a regular
//! UsePowerEffect path) MOVE_CARDS it to the discard; a RetreatEffect with it
//! Active throws; AddSpecialConditionsEffects on it are prevented.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "AntiqueRootFossil",
    mask: mask(&[k::POWER, k::PLAY_ITEM, k::CHECK_ATTACK_COST, k::RETREAT, k::ADD_SPECIAL_CONDITIONS]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let list = match g.st.locate(me) {
            Some(l) => l,
            None => bail!("TypeError: Cannot read properties of undefined"),
        };
        move_cards(g, list, ListRef::Discard(p as u8), &[me], me)?;
    }

    if let Effect::PlayItem { p, card, .. } = *g.e(e) {
        if card == me {
            let pu = p as usize;
            let slots = empty_bench_slots(g, pu);
            let s = match slots.as_slice().first() {
                Some(s) => *s,
                None => bail!("CANNOT_PLAY_THIS_CARD"),
            };
            g.run_fx(Effect::PlayPokemon { p, card: me, target: SlotRef::new(pu, s), slot: SlotType::Board, index: 0 })?;
        }
    }

    if let Effect::CheckAttackCost { p, .. } = *g.e(e) {
        let attacker = p as usize;
        let opponent = 1 - attacker;
        if g.st.active_pokemon(opponent) == Some(me) {
            let basic = g.st.active_pokemon(attacker).map(|c| g.st.cdef(c).stage == Stage::Basic as u8).unwrap_or(false);
            if basic && !is_ability_blocked(g, opponent, me, None) {
                if let Effect::CheckAttackCost { cost, .. } = g.e_mut(e) {
                    cost.push(ct::COLORLESS);
                }
            }
        }
    }

    if let Effect::Retreat { p, .. } = *g.e(e) {
        if g.st.active_pokemon(p as usize) == Some(me) {
            bail!("CANNOT_RETREAT");
        }
    }

    if let Effect::AddSpecialConditions { b, .. } = *g.e(e) {
        if g.st.slot_pokemon(b.target.p as usize, b.target.s) == Some(me) {
            g.set_prevent(e, true);
        }
    }
    Ok(())
}
