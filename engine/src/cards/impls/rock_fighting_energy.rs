//! Rock Fighting Energy (POR, "Rocky Fighting Energy"): provides [F]. Prevent
//! all effects of attacks used by your opponent's Pokémon done to the [F]
//! Pokémon this card is attached to (damage is not an effect).
//!
//! Twinleaf: the [F] entry is pushed on every CheckProvidedEnergyEffect of a
//! slot holding this card. Like Mist Energy, every AbstractAttackEffect on a
//! slot holding this card whose top Pokémon is printed [F] is prevented
//! unless it is ApplyWeakness / PutDamage / DealDamage, when its source slot
//! belongs to the target owner's opponent and holds a Pokémon; the
//! special-energy block probe runs first, for the target owner's opponent.
use crate::cards::prelude::*;
use crate::effects::EnergyEntry;

pub static IMPL: CardImpl = CardImpl {
    class: "RockFightingEnergy",
    mask: mask(&[
        k::CHECK_PROVIDED_ENERGY,
        k::APPLY_WEAKNESS,
        k::DEAL_DAMAGE,
        k::PUT_DAMAGE,
        k::AFTER_DAMAGE,
        k::PUT_COUNTERS,
        k::KNOCK_OUT_OPPONENT,
        k::KNOCK_OUT_PLAYER,
        k::DISCARD_CARDS,
        k::CARDS_TO_HAND,
        k::GUST_OPPONENT_BENCH,
        k::ADD_MARKER,
        k::ADD_SPECIAL_CONDITIONS,
        k::REMOVE_SPECIAL_CONDITIONS,
        k::HEAL_TARGET,
        k::PLAY_LOCK,
        k::PREVENT_RETREAT,
    ]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckProvidedEnergy { source, .. } = *g.e(e) {
        if g.st.slot(source.p as usize, source.s).cards.contains(me) {
            let mut provides = SVec::new();
            provides.push(ct::FIGHTING);
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
                energy_map.push(EnergyEntry { card: me, provides });
            }
        }
        return Ok(());
    }
    let b = match g.e(e).atk_base() {
        Some(b) => *b,
        None => return Ok(()),
    };
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).cards.contains(me) {
        return Ok(());
    }
    let fighting = g.st.slot_pokemon(t.p as usize, t.s).map(|c| g.st.cdef(c).card_type.contains(&ct::FIGHTING)).unwrap_or(false);
    if !fighting {
        return Ok(());
    }
    let opponent = 1 - t.p as usize;
    if is_special_energy_blocked(g, opponent, me, t, false) {
        return Ok(());
    }
    if b.source.p as usize != opponent {
        return Ok(());
    }
    if g.st.slot_pokemon(b.source.p as usize, b.source.s).is_some() {
        if matches!(*g.e(e), Effect::ApplyWeakness { .. } | Effect::PutDamage { .. } | Effect::DealDamage { .. }) {
            return Ok(());
        }
        g.set_prevent(e, true);
    }
    Ok(())
}
