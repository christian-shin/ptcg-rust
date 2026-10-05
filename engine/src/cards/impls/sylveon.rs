//! Sylveon (PRE): Safeguard - prevent all damage done to this Pokémon by
//! attacks from your opponent's Pokémon ex. Magical Shot - 100.
//!
//! Twinleaf: any PutDamageEffect on a slot holding this card (with this card
//! on top) during the ATTACK phase, from another player's ex Pokémon (source
//! top card tagged `POKEMON_ex`), is prevented unless the Ability is blocked.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Sylveon", mask: mask(&[k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::PutDamage { b, .. } => b,
        _ => return Ok(()),
    };
    if ignores_defender_effects(g, &b) {
        return Ok(());
    }
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).cards.contains(me) {
        return Ok(());
    }
    let source_card = g.st.slot_pokemon(b.source.p as usize, b.source.s);
    let player = t.p as usize;
    let opponent = b.source.p as usize;
    if player == opponent || g.st.slot_pokemon(player, t.s) != Some(me) || source_card.is_none() || g.st.phase != GamePhase::Attack {
        return Ok(());
    }
    let ex = g.st.cdef(source_card.unwrap()).has_tag(tag::POKEMON_EX_LOWER);
    if ex && !is_ability_blocked(g, player, me, None) {
        g.set_prevent(e, true);
    }
    Ok(())
}
