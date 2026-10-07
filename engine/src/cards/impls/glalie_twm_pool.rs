//! Glalie (TWM 52): Damage Beat — 20 damage for each damage counter on your
//! opponent's Active Pokémon. Crazy Headbutt — 140; discard an Energy from
//! this Pokémon.
//!
//! Twinleaf: Crazy Headbutt is DISCARD_UP_TO_X_ENERGY_FROM_THIS_POKEMON(1, {}, 1).
use super::team_rockets_houndoom_dri_pool::{discard_up_to_chosen, discard_up_to_x_energy_from_this_pokemon};
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GlalieTWMPool", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[o].active;
        let d = g.st.slot(o, a).damage;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 20 * (d / 10);
        }
    }
    if after_attack_used(g, e, 1, me) {
        let e = real_attack(g, e);
        return discard_up_to_x_energy_from_this_pokemon(g, me, e, 1, 1, 1);
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_up_to_chosen(g, f, results);
    }
    Ok(())
}
