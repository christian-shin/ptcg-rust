//! Shuppet (PBL / M5): Hide 'n' Sneak — prevent all effects of your
//! opponent's Pokémon's attacks and Abilities done to this Pokémon (damage is
//! not an effect). Hang Down — 10.
//!
//! Also hosts the shared `hide-n-sneak.ts` helpers (Banette, Poltchageist,
//! Sinistcha, Dhelmise).
//!
//! Twinleaf (`reduceHideNSneak`): every AbstractAttackEffect whose target
//! slot has this card on top (in play) is prevented unless it is
//! ApplyWeakness / PutDamage / DealDamage, when its `player` is not the
//! owner; the ability-lock probe (for the owner) runs before the owner check.
//! PlaceDamageCountersEffect is prevented when its source card sits in a slot
//! owned by the effect's player. (AddSpecialConditionsPowerEffect and
//! PutDamageCountersEffect branches: no ported card emits them yet.)
use crate::cards::prelude::*;

pub const HIDE_N_SNEAK_KINDS: [u32; 27] = [
    k::SELF_PREVENT_RETREAT,
    k::DISCARD_ATTACKER_ENERGY_IF_KO,
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
    k::OPPONENT_POKEMON_CANNOT_USE_ATTACK,
    k::REDUCE_DAMAGE,
    k::SWITCH_OUT_OPPONENTS_ACTIVE,
    k::PLACE_DAMAGE_COUNTERS,
    k::PREVENT_DAMAGE,
    k::PREVENT_EFFECTS_OF_ATTACKS,
    k::THIS_POKEMON_HAS_NO_WEAKNESS,
    k::RETALIATE_ON_DAMAGE,
    k::RETALIATE_DAMAGE,
];

pub static IMPL: CardImpl = CardImpl { class: "Shuppet", mask: mask(&HIDE_N_SNEAK_KINDS), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    reduce_hide_n_sneak(g, me, e)
}

/// `countHideNSneakPokemonInDiscard(player)`.
pub fn count_hide_n_sneak_in_discard(g: &Game, p: usize) -> usize {
    g.st.players[p]
        .discard
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_pokemon() && d.powers.iter().any(|pw| pw.power_type == PowerType::Ability as u8 && pw.name == "Hide 'n' Sneak")
        })
        .count()
}

/// Target slot has `me` on top and is in its owner's Active/Bench.
fn guarded(g: &Game, me: CardId, t: SlotRef) -> bool {
    let (p, s) = (t.p as usize, t.s);
    if !g.st.slot(p, s).cards.contains(me) || g.st.slot_pokemon(p, s) != Some(me) {
        return false;
    }
    let pl = &g.st.players[p];
    pl.active == s || pl.bench.contains(&s)
}

/// `reduceHideNSneak(store, state, effect, this)`.
pub fn reduce_hide_n_sneak(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(b) = g.e(e).atk_base().copied() {
        let t = b.target;
        if !guarded(g, me, t) {
            return Ok(());
        }
        let owner = t.p as usize;
        if is_ability_blocked(g, owner, me, None) {
            return Ok(());
        }
        if b.player as usize == owner {
            return Ok(());
        }
        if matches!(*g.e(e), Effect::ApplyWeakness { .. } | Effect::PutDamage { .. } | Effect::DealDamage { .. }) {
            return Ok(());
        }
        g.set_prevent(e, true);
        return Ok(());
    }
    if let Effect::PlaceDamageCounters { p, target, source, .. } = *g.e(e) {
        if !guarded(g, me, target) {
            return Ok(());
        }
        let owner = target.p as usize;
        if is_ability_blocked(g, owner, me, None) {
            return Ok(());
        }
        if p as usize == owner {
            return Ok(());
        }
        if source == NO_CARD {
            return Ok(());
        }
        match g.st.find_pokemon_slot(source) {
            Some((q, _)) if q == p as usize => {}
            _ => return Ok(()),
        }
        g.set_prevent(e, true);
    }
    Ok(())
}
