//! Onix (M1L / MEG 70): Bind — 30; flip a coin, if heads the opponent's
//! Active Pokémon is now Paralyzed. Strength — 100.
//!
//! Twinleaf flips on AFTER_ATTACK and paralyzes through
//! ADD_PARALYZED_TO_PLAYER_ACTIVE (an AddSpecialConditionsPowerEffect, not an
//! attack effect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Onix@Onix M1L", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::AfterAttack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        g.coin_flip(p, CoinCb::Card { card: me, frame: f })?;
    }
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    if heads {
        let o = 1 - f.a[0] as usize;
        add_special_conditions_to_player_active(g, o, me, &[SpecialCondition::Paralyzed])?;
    }
    Ok(())
}
