//! Milotic ex (SSP): Sparkling Scales — prevent all damage and effects done
//! to this Pokémon by your opponent's Tera Pokémon's attacks. Hypno Splash —
//! 160; your opponent's Active Pokémon is now Asleep.
//!
//! Twinleaf: the Ability reacts to every AbstractAttackEffect whose target
//! list holds this card (top Pokémon must be this card; source slot must hold
//! a Pokémon; different owners; attack phase; Tera source; generic lock probe
//! for the target owner), setting preventDefault. PutDamageEffect is handled
//! a second time (probe again) and also zeroes `damage`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Miloticex",
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
        k::INCREASE_ATTACK_COST_NEXT_TURN,
        k::INCREASE_RETREAT_COST_NEXT_TURN,
        k::COIN_FLIP_CANCEL_TRAINER_PLAY,
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
    if let Some(b) = g.e(e).atk_base().copied() {
        let t = b.target;
        // Shred: the damage steps are not blocked (see `isDamageIgnoringDefenderEffects`).
        let shred_damage = matches!(*g.e(e), Effect::DealDamage { .. } | Effect::ApplyWeakness { .. } | Effect::PutDamage { .. } | Effect::AfterDamage { .. }) && ignores_defender_effects(g, &b);
        if !shred_damage && g.st.slot(t.p as usize, t.s).cards.contains(me) {
            let is_put = matches!(*g.e(e), Effect::PutDamage { .. });
            'blk: {
                if g.st.slot_pokemon(t.p as usize, t.s) != Some(me) {
                    break 'blk;
                }
                let source_card = match g.st.slot_pokemon(b.source.p as usize, b.source.s) {
                    Some(c) => c,
                    None => break 'blk,
                };
                if t.p == b.source.p {
                    break 'blk;
                }
                if g.st.phase != GamePhase::Attack {
                    break 'blk;
                }
                if g.st.cdef(source_card).has_tag(tag::POKEMON_TERA) {
                    if is_ability_blocked(g, t.p as usize, me, None) {
                        break 'blk;
                    }
                    g.set_prevent(e, true);
                    if is_put {
                        if is_ability_blocked(g, t.p as usize, me, None) {
                            break 'blk;
                        }
                        if let Effect::PutDamage { damage, .. } = g.e_mut(e) {
                            *damage = 0;
                        }
                        g.set_prevent(e, true);
                    }
                }
            }
        }
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Asleep])?;
    }
    Ok(())
}
