//! Power Protein (M1L, item; Twinleaf name "Premium Power Pro"): during this
//! turn, your [F] Pokémon's attacks do 30 more damage to your opponent's
//! Active Pokémon (before applying Weakness and Resistance).
//!
//! Twinleaf: a per-instance player marker; every DealDamageEffect whose
//! player's Active is a (printed) [F] Pokémon, with damage > 0 and the
//! opponent's Active as target, gets +30 per marked copy (the discarded
//! card keeps reacting). Markers are removed at any EndTurn of their holder.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "PowerProtein",
    mask: mask(&[k::TRAINER, k::DEAL_DAMAGE, k::END_TURN]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn protein() -> crate::markers::MarkerName {
    crate::marker!("POWER_PROTEIN_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        g.st.players[p].marker.add(protein(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    }
    if let Effect::DealDamage { b, damage } = *g.e(e) {
        let p = b.player as usize;
        let fighting = g.st.active_pokemon(p).map(|c| g.st.cdef(c).card_type.contains(&ct::FIGHTING)).unwrap_or(false);
        if fighting {
            let o = 1 - p;
            let t = b.target;
            if g.st.players[p].marker.has_from(protein(), me) && damage > 0 && t.p as usize == o && t.s == g.st.players[o].active {
                if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                    *damage += 30;
                }
            }
        }
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].marker.remove_from(protein(), me);
    }
    Ok(())
}
