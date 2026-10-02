//! Light Ball (ASC, tool): attacks used by the Pikachu ex this card is
//! attached to do 50 more damage to your opponent's Active Pokémon ex
//! (before applying Weakness and Resistance).
//!
//! Twinleaf: on DealDamageEffect from the holder's slot: the holder must be
//! named 'Pikachu ex', then the tool probe, then the target must be the
//! opponent's Active; +50 when that is an ex and the current damage is
//! above 0.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LightBall", mask: mask(&[k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (b, damage) = match *g.e(e) {
        Effect::DealDamage { b, damage } => (b, damage),
        _ => return Ok(()),
    };
    if !g.st.slot(b.source.p as usize, b.source.s).tools.contains(me) {
        return Ok(());
    }
    let p = b.player as usize;
    let o = 1 - p;
    let pikachu = g.st.slot_pokemon(b.source.p as usize, b.source.s).map(|c| g.st.cdef(c).name == "Pikachu ex").unwrap_or(false);
    if !pikachu {
        return Ok(());
    }
    if g.run_fx(Effect::Tool { p: p as u8, card: me }).is_err() {
        return Ok(());
    }
    let t = b.target;
    if !(t.p as usize == o && t.s == g.st.players[o].active) {
        return Ok(());
    }
    let ex = g.st.slot_pokemon(t.p as usize, t.s).map(|c| g.st.cdef(c).has_tag(tag::POKEMON_EX_LOWER)).unwrap_or(false);
    if ex && damage > 0 {
        if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
            *damage += 50;
        }
    }
    Ok(())
}
