//! Team Rocket's Porygon2 (DRI): R Command — 20 damage for each Supporter
//! with "Team Rocket" in its name in your discard pile.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsPorygon2", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let n = g.st.players[p]
        .discard
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_trainer() && d.name.contains("Team Rocket") && d.trainer_type == TrainerType::Supporter as u8
        })
        .count() as i32;
    if let Effect::Attack { damage, .. } = g.e_mut(e) {
        *damage = 20 * n;
    }
    Ok(())
}
