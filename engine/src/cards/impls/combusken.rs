//! Combusken (DRI): Combustion — 20. Double Kick — 40x; flip 2 coins
//! (`effect.damage = 40 * heads`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Combusken", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

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

fn resume(g: &mut Game, _me: CardId, f: CardFrame, _results: &[Res]) -> R {
    if f.stage == 1 {
        let atk = f.e[0];
        let heads = (f.a[2] as u32).count_ones() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage = 40 * heads;
        }
        g.release_fx(atk);
    }
    Ok(())
}
