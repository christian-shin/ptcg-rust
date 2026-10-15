//! Team Rocket's Houndoom (DRI 38): Cruel Coal: your opponent's Active Pokémon is now Burned and Confused. Scorching Fire:
//! 120; discard an Energy from this Pokémon.
//!
//! Cruel Coal gives the two Special Conditions (a GainCondition event each, this attack its cause); Scorching Fire discards
//! one Energy after the damage (a LeavePlay event, this attack its cause).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsHoundoom",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Burned, SpecialCondition::Confused]))] },
        AttackSpec {
            index: 1,
            // Discard an Energy from this Pokémon.
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Scoped { scope: PromptScope::Active, min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, clamp: true }, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
