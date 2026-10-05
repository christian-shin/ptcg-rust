//! Payapa Berry (SCR 141, Pokémon Tool): if the Pokémon this card is attached
//! to is damaged by an attack from your opponent's [P] Pokémon, it takes 60
//! less damage (after applying Weakness and Resistance), and discard this
//! card.
//!
//! Twinleaf (shared with Babiri Berry): on a PutDamageEffect whose target
//! holds this tool, during the attack phase, unless the tool is blocked for
//! the owner or the source belongs to the same player: a CheckPokemonType on
//! the source; if it contains the type, the damage is reduced by 60 (floored
//! at 0) and the tool moves from the target slot to the owner's discard (no
//! sourceCard), even when the damage was already 0.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PayapaBerrySCRPool", mask: mask(&[k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

/// The shared Haban-style berry logic with the attacker type and reduction.
pub fn berry_reduce(g: &mut Game, me: CardId, e: EffId, attacker_type: CardType) -> R {
    let b = match *g.e(e) {
        Effect::PutDamage { b, .. } => b,
        _ => return Ok(()),
    };
    if ignores_defender_effects(g, &b) {
        return Ok(());
    }
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
        return Ok(());
    }
    if g.st.phase != GamePhase::Attack {
        return Ok(());
    }
    let owner = t.p as usize;
    if is_tool_blocked(g, owner, me) {
        return Ok(());
    }
    if b.source.p == t.p {
        return Ok(());
    }
    let types = crate::engine::game_effect::pokemon_types(g, b.source);
    let (ct_e, _) = g.run_fx(Effect::CheckPokemonType { target: b.source, card_types: types })?;
    if !matches!(ct_e, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&attacker_type)) {
        return Ok(());
    }
    if let Effect::PutDamage { damage, .. } = g.e_mut(e) {
        *damage = (*damage - 60).max(0);
    }
    move_cards(g, ListRef::Slot(t.p, t.s), ListRef::Discard(owner as u8), &[me], NO_CARD)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    berry_reduce(g, me, e, ct::PSYCHIC)
}
