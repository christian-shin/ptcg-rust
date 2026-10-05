//! Team Rocket's Articuno (DRI): Repelling Veil — prevent all effects of the
//! opponent's attacks done to your Basic Team Rocket's Pokémon. Dark Frost —
//! 60+; 60 more if this Pokémon has Team Rocket's Energy attached.
//!
//! Twinleaf (fixed in phase 4b, R1-2): Repelling Veil is Mist Energy's
//! pattern for the owner's Basic Team Rocket's Pokémon: every
//! AbstractAttackEffect whose target slot holds a Basic Team Rocket's Pokémon
//! is prevented, unless it is ApplyWeakness / PutDamage / DealDamage, when
//! this Articuno is in play on the target owner's side, its Ability isn't
//! blocked (probe for the owner), the effect comes from the owner's
//! opponent's Pokémon and the source slot holds a Pokémon. (It used to
//! prevent only PutCountersEffect, with no Ability-lock check.) Dark Frost
//! looks for an Energy named "Team Rocket's Energy" on the attacker's Active
//! (fixed in phase 4b: it compared against "Team Rocket Energy", so the bonus
//! never applied).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "TeamRocketsArticuno",
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
        k::PREVENT_ATTACK_UNTIL_LEAVES_ACTIVE,
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
        k::ATTACK,
    ]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    // Repelling Veil (every AbstractAttackEffect).
    if let Some(b) = g.e(e).atk_base().copied() {
        let t = b.target;
        let top = match g.st.slot_pokemon(t.p as usize, t.s) {
            Some(c) => c,
            None => return Ok(()),
        };
        {
            let d = g.st.cdef(top);
            if d.stage != Stage::Basic as u8 || !d.has_tag(tag::TEAM_ROCKET) {
                return Ok(());
            }
        }
        let owner = t.p as usize;
        if !for_each_pokemon(g, owner, PlayerType::BottomPlayer).iter().any(|(_, c, _)| *c == me) {
            return Ok(());
        }
        if is_ability_blocked(g, owner, me, None) {
            return Ok(());
        }
        // IS_ATTACK_EFFECT_FROM_OPPONENTS_POKEMON
        if b.source.p as usize != 1 - owner {
            return Ok(());
        }
        if g.st.slot_pokemon(b.source.p as usize, b.source.s).is_some() {
            if matches!(*g.e(e), Effect::ApplyWeakness { .. } | Effect::PutDamage { .. } | Effect::DealDamage { .. }) {
                return Ok(());
            }
            g.set_prevent(e, true);
        }
        return Ok(());
    }
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let has = g.st.slot(p, a).cards.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.name == "Team Rocket's Energy"
        });
        if has {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 60;
            }
        }
    }
    Ok(())
}
