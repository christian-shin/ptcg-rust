//! Team Rocket's Ekans (DRI): Hold Back — flip a coin; if heads, the
//! opponent's Active is now Paralyzed (on AfterAttackEffect). Gnaw — 10.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsEkans", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        if let Effect::AfterAttack { p, .. } = *g.e(e) {
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            g.coin_flip(p as usize, CoinCb::Card { card: me, frame: f })?;
        }
    }
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    if heads {
        let p = f.a[0] as usize;
        add_special_conditions_to_player_active(g, 1 - p, me, &[SpecialCondition::Paralyzed])?;
    }
    Ok(())
}
