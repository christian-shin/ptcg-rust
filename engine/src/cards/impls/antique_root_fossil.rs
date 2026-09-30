//! Antique Root Fossil (SCR): play this card as a 60 HP Basic [C] Pokémon.
//! It can't be affected by Special Conditions and can't retreat. At any time
//! during your turn, you may discard it from play.
//!
//! Twinleaf: on its own PlayItemEffect the card reduces a PlayPokemonEffect
//! into the first empty Bench slot (the item play then continues and finds
//! the card no longer in hand). In play, its Trainer Ability (a regular
//! UsePowerEffect path) MOVE_CARDS it to the discard; a RetreatEffect with it
//! Active throws; AddSpecialConditionsEffects on it are prevented.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "AntiqueRootFossil",
    mask: mask(&[k::POWER, k::PLAY_ITEM, k::RETREAT, k::ADD_SPECIAL_CONDITIONS]),
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
