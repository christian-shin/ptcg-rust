//! Team Rocket's Houndoom (DRI 38): Cruel Coal — your opponent's Active
//! Pokémon is now Burned and Confused. Scorching Fire — 120; discard an
//! Energy from this Pokémon.
//!
//! Rule: Cruel Coal is two GainConditions (Burned, Confused) with the attack's cause.
//! Scorching Fire: DISCARD_UP_TO_X_ENERGY_FROM_THIS_POKEMON(1, {},
//! 1): no prompt without Energy on the Active; otherwise a non-cancellable
//! DiscardEnergyPrompt (min = min(1, available), max = min(1, available)),
//! then one DiscardCardsEffect per source slot. Shared by Galarian Obstagoon
//! and Iron Boulder ex.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsHoundoomDRIPool",
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
