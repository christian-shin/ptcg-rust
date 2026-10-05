//! Sacred Charm (PFL; Twinleaf "Sacred Charm M2", tool): the Pokémon this card
//! is attached to takes 30 less damage from attacks from your opponent's
//! Pokémon that have an Ability (after applying Weakness and Resistance).
//!
//! Twinleaf (fixed in phase 4b): on a PutDamageEffect whose target holds this
//! tool, during the attack phase, unless the tool is blocked for the owner or
//! the source belongs to the same player: a CheckPokemonPowersEffect on the
//! source's Pokémon (the old handler tested `effect.source instanceof
//! PokemonCard`, but the source is a PokemonCardList, so it never applied);
//! if any power is an Ability, the damage is reduced by 30 (floored at 0).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SacredCharm", mask: mask(&[k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
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
    let attacker = b.source.p as usize;
    if owner == attacker {
        return Ok(());
    }
    let src = match g.st.slot_pokemon(attacker, b.source.s) {
        Some(c) => c,
        None => return Ok(()),
    };
    let mut powers = SVec::new();
    for i in 0..g.st.cdef(src).powers.len() {
        powers.push(PowerRef { card: src, index: i as u8 });
    }
    let (pe, _) = g.run_fx(Effect::CheckPokemonPowers { p: attacker as u8, target: src, powers })?;
    let has_ability = match pe {
        Effect::CheckPokemonPowers { powers, .. } => powers.iter().any(|r| g.st.cdef(r.card).powers[r.index as usize].power_type == PowerType::Ability as u8),
        _ => false,
    };
    if has_ability {
        if let Effect::PutDamage { damage, .. } = g.e_mut(e) {
            *damage = (*damage - 30).max(0);
        }
    }
    Ok(())
}
