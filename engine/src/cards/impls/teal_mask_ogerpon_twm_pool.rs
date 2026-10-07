//! Teal Mask Ogerpon (TWM 24): Mountain Stroll — search your deck for up to 2
//! Basic Energy cards, reveal them and put them into your hand, then shuffle.
//! Ogre Comeback — 20+; 20 more damage for each of your opponent's Benched
//! Pokémon.
//!
//! Twinleaf: SEARCH_DECK_FOR_CARDS_TO_HAND with a { superType: ENERGY,
//! energyType: BASIC } filter (shown to the opponent), min 0, max 2, no cancel.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TealMaskOgerponTWMPool", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
        search_deck_for_cards_to_hand(g, p, me, filter, ChooseCardsOpts::new(0, 2, false));
    }
    if was_attack_used(g, e, 1, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let benched = g.st.players[o].bench.iter().filter(|b| !g.st.slot(o, **b).cards.is_empty()).count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += 20 * benched;
        }
    }
    Ok(())
}
