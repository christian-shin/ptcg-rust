//! Team Rocket's Kangaskhan ex (ASC): Comet Punch — flip 4 coins, 30 damage
//! for each heads. Wicked Impact — 120+, 100 more if you played a Team
//! Rocket Supporter from your hand this turn (`player.rocketSupporter`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsKangaskhanex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        if let Err(err) = coin_flip_sequence(g, p, 4, CoinCb::SequenceCard { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].rocket_supporter {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 100;
            }
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, _results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let heads = (f.a[2] as u32).count_ones() as i32;
    if let Effect::Attack { damage, .. } = g.e_mut(atk) {
        *damage = 30 * heads;
    }
    g.release_fx(atk);
    Ok(())
}
