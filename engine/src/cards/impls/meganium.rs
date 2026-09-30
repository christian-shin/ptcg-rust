//! Meganium (M1S): Wild Growth — each Basic [G] Energy attached to your
//! Pokémon provides [G][G] Energy (only 1 Wild Growth at a time). Solar Beam — 140.
//!
//! Twinleaf: on every CheckProvidedEnergyEffect whose player has this card
//! as the top card of an in-play Pokémon (and the ability isn't blocked),
//! every Energy card in the source's `cards` providing [G] (basic or not)
//! not yet in the map gets a [G][G] entry. Extra copies find their entries
//! already mapped.
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl { class: "Meganium", mask: mask(&[k::CHECK_PROVIDED_ENERGY]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, source) = match *g.e(e) {
        Effect::CheckProvidedEnergy { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    if !for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| *c == me) {
        return Ok(());
    }
    if is_ability_blocked(g, p, me, None) {
        return Ok(());
    }
    let cards: Vec<CardId> = g.st.slot(source.p as usize, source.s).cards.iter().collect();
    for c in cards {
        let d = g.st.cdef(c);
        if !d.is_energy() || !d.provides.contains(&ct::GRASS) {
            continue;
        }
        if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
            if energy_map.iter().any(|m| m.card == c) {
                continue;
            }
            let mut provides = SVec::new();
            provides.push(ct::GRASS);
            provides.push(ct::GRASS);
            energy_map.push(EnergyEntry { card: c, provides });
        }
    }
    Ok(())
}
