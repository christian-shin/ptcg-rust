//! Empoleon ex (PFL 70): Emperor's Stance — prevent all effects of attacks
//! used by your opponent's Pokémon done to this Pokémon (damage is not an
//! effect). Iron Feathers — 210; during your opponent's next turn this
//! Pokémon takes 60 less damage from attacks.
//!
//! Twinleaf: every AbstractAttackEffect whose target list holds this card
//! runs the ability-lock probe (for the effect's player) first; then it is
//! ignored when source and target have the same owner, and otherwise
//! prevented unless it is ApplyWeakness / PutDamage / DealDamage, when the
//! source slot holds a Pokémon. Iron Feathers sets
//! `player.active.damageReductionNextTurn = 60` directly.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Empoleonex",
    mask: mask(&[
        k::ATTACK,
        k::APPLY_WEAKNESS,
        k::DEAL_DAMAGE,
        k::PUT_DAMAGE,
        k::AFTER_DAMAGE,
        k::PUT_COUNTERS,
        k::KNOCK_OUT_OPPONENT,
        k::DISCARD_CARDS,
        k::CARDS_TO_HAND,
        k::GUST_OPPONENT_BENCH,
        k::MOVE_OPPONENT_ENERGY,
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
        k::OPPONENT_POKEMON_CANNOT_ATTACK_NEXT_TURN,
        k::RETALIATE_ON_DAMAGE,
        k::RETALIATE_DAMAGE,
        k::MOVE_COUNTERS,
    ]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].damage_reduction_next_turn = 60;
        }
    }
    let b = match g.e(e).atk_base() {
        Some(b) => *b,
        None => return Ok(()),
    };
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).cards.contains(me) {
        return Ok(());
    }
    if is_ability_blocked(g, b.player as usize, me, None) {
        return Ok(());
    }
    if b.source.p == t.p {
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
