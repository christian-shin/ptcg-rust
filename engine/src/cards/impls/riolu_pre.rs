//! Riolu (PRE): Quick Attack — 10+; flip a coin, if heads 20 more damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Riolu@PRE",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(20, Cond::True))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// Helpers still imported by hand-written cards (Bronzor, Comfey, Crustle, Eevee, Ion's Wattrel);
// delete when their conversions land.
use crate::cards::prelude::*;

pub fn flip_more_damage(g: &mut Game, me: CardId, e: EffId, amount: i32) -> R {
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    g.retain_fx(e);
    let mut f = CardFrame::at(0);
    f.e[0] = e;
    f.a[0] = amount;
    if let Err(err) = g.coin_flip(p, CoinCb::Card { card: me, frame: f }) {
        g.release_fx(e);
        return Err(err);
    }
    Ok(())
}

pub fn coin_more_damage(g: &mut Game, f: CardFrame, heads: bool) -> R {
    let atk = f.e[0];
    if heads {
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage += f.a[0];
        }
    }
    g.release_fx(atk);
    Ok(())
}
