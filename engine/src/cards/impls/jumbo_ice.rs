//! Jumbo Ice Cream (PFL / M2): heal 80 damage from your Active Pokémon that
//! has 3 or more Energy attached.
//!
//! Twinleaf: phase 4b (R6): throws CANNOT_PLAY_THIS_CARD when the Active has
//! no damage or fewer than 3 Energy attached (it used to just discard the card
//! with no effect); the Energy count is the number of provided-energy
//! entries; the card moves itself from the supporter pile to the discard.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "JumboIce", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let a = g.st.players[p].active;
    if g.st.slot_pokemon(p, a).is_none() || g.st.slot(p, a).damage == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let src = SlotRef::new(p, a);
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: src, energy_map: SVec::new() })?;
    let n = match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.len(),
        _ => 0,
    };
    if n < 3 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.run_fx(Effect::Heal { p: p as u8, target: src, damage: 80 })?;
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)?;
    Ok(())
}
