//! Galarian Zigzagoon (FST 159, support card): Lick — 10; flip a coin, if
//! heads your opponent's Active Pokémon is now Paralyzed.
//!
//! Twinleaf: after the attack (AfterAttackEffect) a COIN_FLIP_PROMPT; on
//! heads ADD_PARALYZED_TO_PLAYER_ACTIVE for the opponent, i.e. an
//! AddSpecialConditionsPowerEffect (not the attack effect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GalarianZigzagoon@Galarian Zigzagoon FST 159", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        if let Effect::AfterAttack { p, opp, .. } = *g.e(e) {
            let mut f = CardFrame::at(0);
            f.a[0] = opp as i32;
            g.coin_flip(p as usize, CoinCb::Card { card: me, frame: f })?;
        }
    }
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    if heads {
        add_special_conditions_to_player_active(g, f.a[0] as usize, me, &[SpecialCondition::Paralyzed])?;
    }
    Ok(())
}
