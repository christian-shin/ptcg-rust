//! Comfey (SCR): Flower Shower — each player draws 3 cards (MOVE_CARDS
//! count 3, so fewer cards simply move fewer; phase 4b: it threw
//! CANNOT_USE_ATTACK when either deck was empty). Play Rough — 20+; flip a
//! coin, if heads 20 more damage.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Comfey", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p, opp),
            _ => return Ok(()),
        };
        move_count_from(g, ListRef::Deck(p), ListRef::Hand(p), 3, me)?;
        move_count_from(g, ListRef::Deck(o), ListRef::Hand(o), 3, me)?;
    }
    if was_attack_used(g, e, 1, me) {
        super::riolu_pre::flip_more_damage(g, me, e, 20)?;
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    super::riolu_pre::coin_more_damage(g, f, heads)
}
