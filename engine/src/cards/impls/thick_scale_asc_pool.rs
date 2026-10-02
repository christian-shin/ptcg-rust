//! Thick Scale (ASC 211, Pokémon Tool): the [N] Pokémon this card is attached
//! to takes 50 less damage from attacks from your opponent's [G], [R], [W],
//! or [L] Pokémon (after applying Weakness and Resistance).
//!
//! Twinleaf: on a PutDamageEffect whose target holds this tool, during the
//! attack phase, unless the tool is blocked for the owner or the source
//! belongs to the same player: a CheckPokemonType on the holder must contain
//! [N], then one on the source must contain [G], [R], [W] or [L]; the damage is
//! reduced by 50 (floored at 0). The card stays attached.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ThickScaleASCPool", mask: mask(&[k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::PutDamage { b, .. } => b,
        _ => return Ok(()),
    };
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
        return Ok(());
    }
    if g.st.phase != GamePhase::Attack {
        return Ok(());
    }
    if is_tool_blocked(g, t.p as usize, me) {
        return Ok(());
    }
    if b.source.p == t.p {
        return Ok(());
    }
    let types = crate::engine::game_effect::pokemon_types(g, t);
    let (holder, _) = g.run_fx(Effect::CheckPokemonType { target: t, card_types: types })?;
    if !matches!(holder, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::DRAGON)) {
        return Ok(());
    }
    let types = crate::engine::game_effect::pokemon_types(g, b.source);
    let (src, _) = g.run_fx(Effect::CheckPokemonType { target: b.source, card_types: types })?;
    let ok = matches!(src, Effect::CheckPokemonType { card_types, .. }
        if card_types.iter().any(|c| *c == ct::GRASS || *c == ct::FIRE || *c == ct::WATER || *c == ct::LIGHTNING));
    if !ok {
        return Ok(());
    }
    if let Effect::PutDamage { damage, .. } = g.e_mut(e) {
        *damage = (*damage - 50).max(0);
    }
    Ok(())
}
