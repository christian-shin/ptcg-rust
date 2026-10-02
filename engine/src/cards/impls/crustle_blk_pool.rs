//! Crustle (BLK): Sturdy - if this Pokémon has full HP and would be Knocked
//! Out by damage from an attack, it is not Knocked Out and its remaining HP
//! becomes 10 (SURVIVE_ON_TEN_IF_FULL_HP, see `crustle_bcr.rs`). Stone Edge -
//! 80+; flip a coin, if heads 60 more damage.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "CrustleBLKPool", mask: mask(&[k::PUT_DAMAGE, k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    super::crustle_bcr::survive_on_ten_if_full_hp(g, me, e)?;
    if was_attack_used(g, e, 0, me) {
        super::riolu_pre::flip_more_damage(g, me, e, 60)?;
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    super::riolu_pre::coin_more_damage(g, f, heads)
}
