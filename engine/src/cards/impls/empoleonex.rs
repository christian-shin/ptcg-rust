//! Empoleon ex (PFL 70): Emperor's Stance — prevent all effects of attacks used by your opponent's Pokémon done to this
//! Pokémon (damage is not an effect). Iron Feathers — 210; during your opponent's next turn this Pokémon takes 60 less
//! damage from attacks.
//!
//! Emperor's Stance is one `Prevent` naming no kind (`EFFECTS_OF_OPP_ATTACKS`): it ranges over every event with an
//! effect the opponent's attacks cause to this Pokémon, never Damage (APR C-17, id2333: damage added by an effect is
//! not blocked). Iron Feathers arms
//! `Lasting::TakesLessDamage(60)` (one ApplyEffect event), applied after Weakness and Resistance.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Empoleonex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(60) }))] }],
    passives: &[
        // Emperor's Stance: every event the opponent's attacks cause to this Pokémon, the switches included (APR
        // C-04 / C-05, id2025, id2155); damage is not an effect.
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EFFECTS_OF_OPP_ATTACKS)) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
