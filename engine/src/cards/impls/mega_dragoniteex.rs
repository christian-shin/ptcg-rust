//! Mega Dragonite ex (M2a / ASC 152): Sky Transport — once during your turn,
//! switch your Active Pokémon with 1 of your Benched Pokémon. Ryuno Glide —
//! 330; DISCARD_X_ENERGY_FROM_THIS_POKEMON(2).
//!
//! Twinleaf order: ability-blocked probe, marker check, bench check, then
//! SWITCH_ACTIVE_WITH_BENCHED (prompt), marker, board effect.
use crate::cards::prelude::*;
use crate::cards::registry::slither_wing::{discard_energy_chosen, discard_x_energy_from_this_pokemon};

pub static IMPL: CardImpl = CardImpl {
    class: "MegaDragoniteex",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::POWER, k::ATTACK, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn marker() -> crate::markers::MarkerName {
    crate::marker!("SKY_CARRY_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(marker(), me);
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(marker(), me) {
            m.remove_from(marker(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        if g.st.players[p].marker.has_from(marker(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let pl = &g.st.players[p];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            bail!("CANNOT_USE_POWER");
        }
        switch_active_with_benched(g, p);
        g.st.players[p].marker.add(marker(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        ability_used(g, p, me);
    }

    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        discard_x_energy_from_this_pokemon(g, me, e, 2, 1)?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_energy_chosen(g, f, results);
    }
    Ok(())
}
