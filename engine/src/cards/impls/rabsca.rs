//! Rabsca (TEF): Spherical Shield — prevent all damage from and effects of
//! attacks done to your Benched Pokémon by your opponent's attacks.
//! Psychic — 10+; 30 more for each Energy attached to the opponent's Active.
//!
//! Twinleaf: PREVENT_DAMAGE_/PREVENT_EFFECTS_TO_YOUR_BENCHED_POKEMON_FROM_
//! OPPONENT_ATTACKS for each player as owner (includeSourcePokemon, so
//! Rabsca itself is protected too). The lock probe runs before the target
//! check whenever Rabsca's owner sees an attack sub-effect. Psychic counts
//! the opponent's provided energy entries' `provides` lengths.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Rabsca",
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
    ]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn in_play(g: &Game, owner: usize, me: CardId) -> bool {
    for_each_pokemon(g, owner, PlayerType::BottomPlayer).iter().any(|(_, c, _)| *c == me)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp,
            _ => return Ok(()),
        };
        let o = opp as usize;
        let src = SlotRef::new(o, g.st.players[o].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: opp, source: src, energy_map: SVec::new() })?;
        let n: i32 = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|m| m.provides.len() as i32).sum(),
            _ => 0,
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += n * 30;
        }
    }

    let b = match g.e(e).atk_base() {
        Some(b) => *b,
        None => return Ok(()),
    };
    let is_damage = matches!(*g.e(e), Effect::PutDamage { .. } | Effect::PutCounters { .. });
    let damage_like = matches!(*g.e(e), Effect::DealDamage { .. } | Effect::PutDamage { .. } | Effect::PutCounters { .. } | Effect::ApplyWeakness { .. } | Effect::AfterDamage { .. });
    for owner in 0..2usize {
        // PREVENT_DAMAGE_... then PREVENT_EFFECTS_..., each with its own checks.
        for pass in 0..2 {
            let applies = if pass == 0 { is_damage } else { !damage_like };
            if !applies {
                continue;
            }
            if !in_play(g, owner, me) {
                continue;
            }
            if is_ability_blocked(g, owner, me, None) {
                continue;
            }
            let t = b.target;
            let on_bench = t.p as usize == owner && g.st.players[owner].bench.iter().any(|s| *s == t.s);
            if !on_bench {
                continue;
            }
            if b.source.p as usize == owner {
                continue;
            }
            g.set_prevent(e, true);
        }
    }
    Ok(())
}
