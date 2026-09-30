//! Frogadier (TWM): Numbing Water — flip a coin, if heads the opponent's
//! Active Pokémon is now Paralyzed (AddSpecialConditionsEffect from the
//! coin callback).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Frogadier@Frogadier TWM", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(0);
        f.e[0] = e;
        if let Err(err) = g.coin_flip(p, CoinCb::Card { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    let atk = f.e[0];
    let r = if heads { add_special_conditions_to_opponent_active(g, atk, &[SpecialCondition::Paralyzed]) } else { Ok(()) };
    g.release_fx(atk);
    r
}
