//! Cynthia's Roserade (DRI): Glorious Cheer - attacks from your Cynthia's
//! Pokémon deal 30 more damage to your opponent's Active Pokémon.
//!
//! Twinleaf: every DealDamageEffect of the player with this card in play gets
//! +30 when the source slot's Pokémon has the Cynthia's tag.
//!
//! Fixed (phase 4b, W4): the IS_ABILITY_BLOCKED probe ran but its result was
//! ignored, and the bonus applied to any target; it now stops when the
//! Ability is blocked and only applies to the opponent's Active Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "CynthiasRoserade", mask: mask(&[k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::DealDamage { b, .. } => b,
        _ => return Ok(()),
    };
    let p = b.player as usize;
    if !for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| *c == me) {
        return Ok(());
    }
    let attacking = g.st.slot_pokemon(b.source.p as usize, b.source.s);
    if is_ability_blocked(g, p, me, None) {
        return Ok(());
    }
    if b.target.p as usize != 1 - p || b.target.s != g.st.players[1 - p].active {
        return Ok(());
    }
    if let Some(c) = attacking {
        if g.st.cdef(c).has_tag(tag::CYNTHIAS) {
            if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                *damage += 30;
            }
        }
    }
    Ok(())
}
