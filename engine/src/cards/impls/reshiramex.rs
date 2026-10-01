//! Reshiram ex (SV11W): Slash — 50. Blaze Burst — 130+; 50 more damage for
//! each Prize card your opponent has taken (6 - opponent.getPrizeLeft(), an
//! opponent-side count as written in Twinleaf). Discard an Energy from this Pokémon.
use crate::cards::prelude::*;
use super::slither_wing::{discard_energy_chosen, discard_x_energy_from_this_pokemon};

pub static IMPL: CardImpl = CardImpl { class: "Reshiramex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let taken = 6 - g.st.players[opp].prize_left() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += 50 * taken;
        }
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
