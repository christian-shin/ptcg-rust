//! Prism Energy (NXD): provides [C]; on a Basic Pokémon it provides every
//! type of Energy, 1 at a time.
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl { class: "PrismEnergy", mask: mask(&[k::CHECK_PROVIDED_ENERGY]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let source = match *g.e(e) {
        Effect::CheckProvidedEnergy { source, .. } => source,
        _ => return Ok(()),
    };
    let (sp, ss) = (source.p as usize, source.s);
    if !g.st.slot(sp, ss).cards.contains(me) {
        return Ok(());
    }
    let basic = g.st.slot_pokemon(sp, ss).map(|c| g.st.cdef(c).stage == Stage::Basic as u8).unwrap_or(false);
    if !basic {
        return Ok(());
    }
    let mut provides = SVec::new();
    provides.push(ct::ANY);
    if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
        energy_map.push(EnergyEntry { card: me, provides });
    }
    Ok(())
}
