//! Neo Upper Energy (TEF, ACE SPEC): provides [C]; on a Stage 2 Pokémon it
//! provides every type of Energy but only 2 Energy at a time.
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl { class: "NeoUpperEnergy", mask: mask(&[k::CHECK_PROVIDED_ENERGY]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let source = match *g.e(e) {
        Effect::CheckProvidedEnergy { source, .. } => source,
        _ => return Ok(()),
    };
    let (sp, ss) = (source.p as usize, source.s);
    if !g.st.slot(sp, ss).cards.contains(me) {
        return Ok(());
    }
    let stage2 = g.st.slot_pokemon(sp, ss).map(|c| g.st.cdef(c).stage == Stage::Stage2 as u8).unwrap_or(false);
    let mut provides = SVec::new();
    if stage2 {
        provides.push(ct::ANY);
        provides.push(ct::ANY);
    } else {
        provides.push(ct::COLORLESS);
    }
    if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
        energy_map.push(EnergyEntry { card: me, provides });
    }
    Ok(())
}
