//! Mist Energy (TEF): provides [C]. Prevent all effects of attacks from your
//! opponent's Pokémon done to the Pokémon this card is attached to (damage is
//! not an effect).
//!
//! Twinleaf: every AbstractAttackEffect targeting a slot holding this card is
//! prevented unless it is ApplyWeakness / PutDamage / DealDamage, when its
//! source slot belongs to the target owner's opponent and holds a Pokémon.
//! (AfterDamage, PutCounters, KnockOutOpponent, ... are prevented too.) The
//! special-energy block probe runs first, for the target owner's opponent.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "MistEnergy",
    mask: mask(&[
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
        k::REDUCE_DAMAGE,
        k::SWITCH_OUT_OPPONENTS_ACTIVE,
        k::OPPONENT_POKEMON_CANNOT_USE_ATTACK,
        k::SELF_PREVENT_RETREAT,
        k::DISCARD_ATTACKER_ENERGY_IF_KO,
        k::PREVENT_DAMAGE,
        k::PREVENT_EFFECTS_OF_ATTACKS,
        k::THIS_POKEMON_HAS_NO_WEAKNESS,
        k::RETALIATE_ON_DAMAGE,
        k::RETALIATE_DAMAGE,
    ]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match g.e(e).atk_base() {
        Some(b) => *b,
        None => return Ok(()),
    };
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).cards.contains(me) {
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
