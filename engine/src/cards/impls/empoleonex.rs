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
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Empoleonex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(60) }))] }],
    passives: &[
        // Emperor's Stance: every event the opponent's attacks cause to this Pokémon, the switches included (APR
        // C-04 / C-05, id2025, id2155); damage is not an effect.
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::Holder, EFFECTS_OF_OPP_ATTACKS)) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
