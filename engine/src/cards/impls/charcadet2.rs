//! Charcadet (SSP 33, "Charcadet 2"): Light Punch — 10. Flamethrower — 70;
//! discard an Energy from this Pokémon.
use crate::cards::prelude::*;
use super::slither_wing::{discard_energy_chosen, discard_x_energy_from_this_pokemon};

pub static IMPL: CardImpl = CardImpl { class: "Charcadet2", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 1, me) {
        let e = real_attack(g, e);
        discard_x_energy_from_this_pokemon(g, me, e, 1, 1)?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_energy_chosen(g, f, results);
    }
    Ok(())
}
