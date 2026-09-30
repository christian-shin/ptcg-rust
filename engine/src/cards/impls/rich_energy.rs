//! Enriching Energy (Rich Energy SSP, ACE SPEC): provides [C]; when you
//! attach it from your hand to one of your Pokémon, draw 4 cards.
//!
//! Twinleaf: reacts to every AttachEnergyEffect of this card (after the
//! special-energy block probe), not only attachments from the hand.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "RichEnergy", mask: mask(&[k::ATTACH_ENERGY]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::AttachEnergy { p, card, target } = *g.e(e) {
        if card != me {
            return Ok(());
        }
        if is_special_energy_blocked(g, p as usize, me, target, false) {
            return Ok(());
        }
        draw_cards(g, p as usize, 4)?;
    }
    Ok(())
}
