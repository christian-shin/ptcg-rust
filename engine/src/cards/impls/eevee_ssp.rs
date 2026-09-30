//! Eevee (SSP): Boosted Evolution — as long as this Pokémon is in the Active
//! Spot, it can evolve during your first turn or the turn you play it.
//! Reckless Charge — 30; this Pokémon also does 10 damage to itself.
//!
//! Twinleaf has several `Eevee` classes; this port is bound to SSP.
//! Twinleaf quirks kept: every copy (any zone) adds its
//! EVOLUTIONARY_ADVANTAGE_MARKER to the player of any PlayPokemonEffect and
//! removes it at that player's end of turn (the marker does nothing else).
//! On CheckTableStateEffect, if the active player's Active `cards[0]` is this
//! card and a stub-Ability probe passes, `canEvolve = true` (even when this
//! Eevee has evolved) and, while it is still the top Pokémon, its
//! `pokemonPlayedTurn = turn - 1`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Eevee@SSP|Eevee PRE",
    mask: mask(&[k::ATTACK, k::END_TURN, k::PLAY_POKEMON, k::CHECK_TABLE_STATE]),
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
    if let Effect::CheckTableState { .. } = *g.e(e) {
        let p = g.st.active_player as usize;
        let a = g.st.players[p].active;
        if g.st.slot(p, a).cards.get(0) == Some(me) {
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            g.st.players[p].can_evolve = true;
            let turn = g.st.turn;
            for (s, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if c == me {
                    g.st.players[p].slots[s as usize].pokemon_played_turn = turn - 1;
                }
            }
        }
    }
    Ok(())
}
