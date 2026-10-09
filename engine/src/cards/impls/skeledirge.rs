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
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Skeledirge@SSP",
    // Unaware: prevent all effects of attacks used by the opponent's Pokémon done to this Pokémon.
    passives: &[
        // An opponent's attack switching this Pokémon in or out (ChangeActive: APR C-04 / C-05, id2025, id2155).
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), CHANGE_ACTIVE_BY_OPP_ATTACK)) },Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::PreventAttackEffects(PreventAttackEffectsSpec { subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), ..PreventAttackEffectsSpec::DEFAULT }),
    }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Add(&Num::Lit(60), &Num::Mul(&Num::Add(&Num::BenchCount(Who::Me), &Num::BenchCount(Who::Opp)), &Num::Lit(20)))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
