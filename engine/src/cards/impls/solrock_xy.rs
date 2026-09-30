//! Solrock (XY): Cosmic Spin — 10+, 30 more if Lunatone is on your Bench.
//! Solar Beam — 60.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Solrock@XY", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let active = g.st.players[p].active;
        let has = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(s, c, _)| *s != active && g.st.cdef(*c).name == "Lunatone");
        if has {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 30;
            }
        }
    }
    Ok(())
}
