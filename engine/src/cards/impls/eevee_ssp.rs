//! Eevee (SSP): Boosted Evolution — as long as this Pokémon is in the Active
//! Spot, it can evolve during your first turn or the turn you play it.
//! Reckless Charge — 30; this Pokémon also does 10 damage to itself.
//!
//! Twinleaf has several `Eevee` classes; this port is bound to SSP.
//! Twinleaf quirks kept: every copy (any zone) adds its
//! EVOLUTIONARY_ADVANTAGE_MARKER to the player of any PlayPokemonEffect and
//! removes it at that player's end of turn (the marker does nothing else).
//!
//! Fixed in phase 4b (R4): Boosted Evolution answers the CheckPokemonPlayedTurnEffect of
//! this Eevee's own slot: when it is its owner's Active Pokémon, still this
//! card (not evolved) and a stub-Ability probe passes, the effect gets
//! `pokemonPlayedTurn = turn - 1` and `canEvolveOnFirstTurn = true` (the
//! PlayPokemonEffect first-turn test honours it). It used to write
//! `player.canEvolve = true` on every CheckTableStateEffect while the active
//! player's Active `cards[0]` was this card (even after it evolved), letting
//! every Pokémon of that player evolve on the first turn.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Eevee@SSP|Eevee PRE",
    mask: mask(&[k::ATTACK, k::END_TURN, k::PLAY_POKEMON, k::CHECK_POKEMON_PLAYED_TURN]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn marker() -> crate::markers::MarkerName {
    crate::marker!("EVOLUTIONARY_ADVANTAGE_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
            g.run_fx(Effect::DealDamage { b, damage: 10 })?;
        }
        return Ok(());
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].marker.remove_from(marker(), me);
    }
    if let Effect::PlayPokemon { p, .. } = *g.e(e) {
        g.st.players[p as usize].marker.add(marker(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    }
    if let Effect::CheckPokemonPlayedTurn { p, target, .. } = *g.e(e) {
        let owner = p as usize;
        if target.p as usize == owner && target.s == g.st.players[owner].active && g.st.slot_pokemon(target.p as usize, target.s) == Some(me) {
            if is_ability_blocked(g, owner, me, None) {
                return Ok(());
            }
            let turn = g.st.turn as i32;
            if let Effect::CheckPokemonPlayedTurn { pokemon_played_turn, can_evolve_on_first_turn, .. } = g.e_mut(e) {
                *pokemon_played_turn = turn - 1;
                *can_evolve_on_first_turn = true;
            }
        }
    }
    Ok(())
}
