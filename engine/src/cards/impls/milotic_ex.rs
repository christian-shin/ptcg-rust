//! Milotic ex (SSP): Sparkling Scales — prevent all damage and effects done
//! to this Pokémon by your opponent's Tera Pokémon's attacks. Hypno Splash —
//! 160; your opponent's Active Pokémon is now Asleep.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Miloticex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Asleep], Cause::Attack))] }],
    passives: &[
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::PreventAttackEffects(PreventAttackEffectsSpec { subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), attacker: SlotPred::Tag(crate::types::tag::POKEMON_TERA), ..PreventAttackEffectsSpec::DEFAULT }),
        },
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Zero, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), source: SlotPred::Tag(crate::types::tag::POKEMON_TERA), ..PreventDamageSpec::DEFAULT }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
