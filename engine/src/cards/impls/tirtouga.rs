//! Tirtouga (SV11B): Ancient Debris — 30x for each Item card in your
//! opponent's discard pile (`effect.damage = items * 30`). Surf — 80.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Tirtouga", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let items = g.st.players[opp].discard.iter().filter(|c| {
            let d = g.st.cdef(*c);
            d.is_trainer() && d.trainer_type == TrainerType::Item as u8
        }).count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = items * 30;
        }
    }
    Ok(())
}
