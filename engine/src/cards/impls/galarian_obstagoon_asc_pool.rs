//! Galarian Obstagoon (ASC 132): Scarring Shout — 70x damage counters on the
//! opponent's Active. Punk Smash — 160; discard an Energy from this Pokémon.
//!
//! Twinleaf: `damage = 70 * floor(opponent.active.damage / 10)`;
//! DISCARD_UP_TO_X_ENERGY_FROM_THIS_POKEMON(1, {}, 1).
use crate::cards::prelude::*;
use super::team_rockets_houndoom_dri_pool::{discard_up_to_chosen, discard_up_to_x_energy_from_this_pokemon};

pub static IMPL: CardImpl = CardImpl { class: "GalarianObstagoonASCPool", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[opp].active;
        let counters = g.st.slot(opp, a).damage / 10;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 70 * counters as i32;
        }
    }
    if was_attack_used(g, e, 1, me) {
        discard_up_to_x_energy_from_this_pokemon(g, me, e, 1, 1, 1)?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_up_to_chosen(g, f, results);
    }
    Ok(())
}
