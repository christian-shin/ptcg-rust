//! Maximum Belt (TEF, ACE SPEC tool): the holder's attacks do 50 more damage
//! to the opponent's Active Pokémon ex.
//!
//! Twinleaf: reacts to DealDamageEffect from the holder's slot; the bonus
//! needs the DealDamageEffect's current damage above 0. Tool block probe: a
//! bare ToolEffect for the attacking player.
//!
//! Fixed (phase 4b, W4): the target could be either player's Active (so
//! self-damage to the holder's own ex Active was boosted too); it is now the
//! opponent's Active only.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MaximumBelt", mask: mask(&[k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

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
    if g.run_fx(Effect::Tool { p: p as u8, card: me }).is_err() {
        return Ok(());
    }
    let t = b.target;
    let on_opp_active = t.p as usize == o && t.s == g.st.players[o].active;
    if !on_opp_active {
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
