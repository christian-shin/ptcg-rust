//! Skeledirge (SSP): Unaware — prevent all effects of attacks used by your
//! opponent's Pokémon done to this Pokémon (damage is not an effect).
//! Torcherto — 60+, 20 more damage for each Benched Pokémon (both sides).
//!
//! Twinleaf: Torcherto assigns `effect.damage = 60 + 20 * benched`. Unaware
//! reacts to every AbstractAttackEffect whose target slot holds this card
//! once the target's top Pokémon is this card and the source slot has a
//! Pokémon; after the ability-lock probe (stub Ability for the target's
//! owner) everything but ApplyWeakness / PutDamage / DealDamage is prevented.
//!
//! Fixed (phase 4b, R2): Unaware also prevented the effects of the owner's
//! own attacks (a heal, counters from your own Cofagrigus); it now only
//! applies to attacks of the opponent's Pokémon
//! (IS_ATTACK_EFFECT_FROM_OPPONENTS_POKEMON).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Skeledirge@Skeledirge SSP",
    mask: mask(&[
        k::ATTACK,
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
        k::MOVE_OPPONENT_ENERGY,
        k::ADD_MARKER,
        k::ADD_SPECIAL_CONDITIONS,
        k::REMOVE_SPECIAL_CONDITIONS,
        k::HEAL_TARGET,
        k::PLAY_LOCK,
        k::PREVENT_RETREAT,
        k::PREVENT_ATTACK_UNTIL_LEAVES_ACTIVE,
        k::REDUCE_DAMAGE,
        k::SWITCH_OUT_OPPONENTS_ACTIVE,
        k::OPPONENT_POKEMON_CANNOT_USE_ATTACK,
        k::DEFENDING_POKEMON_TAKES_MORE_DAMAGE,
        k::SELF_PREVENT_RETREAT,
        k::DISCARD_ATTACKER_ENERGY_IF_KO,
        k::PREVENT_DAMAGE,
        k::PREVENT_EFFECTS_OF_ATTACKS,
        k::THIS_POKEMON_HAS_NO_WEAKNESS,
        k::OPPONENT_POKEMON_CANNOT_ATTACK_NEXT_TURN,
        k::INCREASE_ATTACK_COST_NEXT_TURN,
        k::INCREASE_RETREAT_COST_NEXT_TURN,
        k::COIN_FLIP_CANCEL_TRAINER_PLAY,
        k::RETALIATE_ON_DAMAGE,
        k::RETALIATE_DAMAGE,
        k::MOVE_COUNTERS,
    ]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn benched(g: &Game, q: usize) -> i32 {
    let pl = &g.st.players[q];
    pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards.is_empty()).count() as i32
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let total = benched(g, p) + benched(g, o);
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 60 + total * 20;
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
    if g.st.slot_pokemon(t.p as usize, t.s) != Some(me) {
        return Ok(());
    }
    // Only attacks used by the opponent's Pokémon are prevented.
    if b.source.p == t.p {
        return Ok(());
    }
    if g.st.slot_pokemon(b.source.p as usize, b.source.s).is_none() {
        return Ok(());
    }
    if is_ability_blocked(g, t.p as usize, me, None) {
        return Ok(());
    }
    if matches!(*g.e(e), Effect::ApplyWeakness { .. } | Effect::PutDamage { .. } | Effect::DealDamage { .. }) {
        return Ok(());
    }
    g.set_prevent(e, true);
    Ok(())
}
