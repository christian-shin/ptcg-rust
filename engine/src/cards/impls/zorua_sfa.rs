//! Zorua (SFA 31): Stampede — 10. Double Scratch — flip 2 coins, 20 damage
//! for each heads.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Zorua@SFA", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        if let Err(err) = coin_flip_sequence(g, p, 2, CoinCb::SequenceCard { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let heads = (results.first().map_or(0, |r| r.as_int()) as u32).count_ones() as i32;
    if let Effect::Attack { damage, .. } = g.e_mut(atk) {
        *damage = 20 * heads;
    }
    g.release_fx(atk);
    Ok(())
}
