//! Cynthia's Roserade (DRI): Glorious Cheer - attacks from your Cynthia's
//! Pokémon deal 30 more damage to your opponent's Active Pokémon.
//!
//! Twinleaf quirks kept: every DealDamageEffect of the player with this card
//! in play gets +30 when the source slot's Pokémon has the Cynthia's tag;
//! the IS_ABILITY_BLOCKED probe runs but its result is ignored.
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
    let _ = is_ability_blocked(g, p, me, None);
    if let Some(c) = attacking {
        if g.st.cdef(c).has_tag(tag::CYNTHIAS) {
            if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                *damage += 30;
            }
        }
    }
    Ok(())
}
