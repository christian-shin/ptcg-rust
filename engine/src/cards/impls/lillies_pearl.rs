//! Lillie's Pearl (JTG, tool): if the Lillie's Pokémon this card is attached
//! to is Knocked Out by damage from an attack from your opponent's Pokémon,
//! that player takes 1 fewer Prize card.
//!
//! Twinleaf checks the owner's DAMAGE_DEALT_MARKER and the slot's Lillie's
//! tag on any card in the slot; the reduction applies before the ex bonus.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LilliesPearl", mask: mask(&[k::KNOCK_OUT]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, target) = match *g.e(e) {
        Effect::KnockOut { p, target, .. } => (p as usize, target),
        _ => return Ok(()),
    };
    if !g.st.slot(target.p as usize, target.s).tools.contains(me) || g.knocked_out_by_attack_damage(p, target).is_none() {
        return Ok(());
    }
    if is_tool_blocked(g, p, me) {
        return Ok(());
    }
    let lillies = g.st.slot(target.p as usize, target.s).cards.iter().any(|c| g.st.cdef(c).has_tag(tag::LILLIES));
    if lillies {
        if let Effect::KnockOut { prize_count, .. } = g.e_mut(e) {
            *prize_count -= 1;
        }
    }
    Ok(())
}
