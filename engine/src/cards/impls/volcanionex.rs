//! Volcanion ex (JTG): Scorching Steam — once during your turn, if this
//! Pokémon is Active, your opponent's Active Pokémon is now Burned. Heat
//! Cyclone — 160; move an Energy from this Pokémon to 1 of your Benched
//! Pokémon.
//!
//! Rule: Scorching Steam's Burn is an Ability effect: GainCondition(Burned) on
//! the opponent's Active with the Ability's cause (Ability locks and Prevent
//! apply). Heat Cyclone's
//! AttachEnergyPrompt targets your own Benched Pokémon (phase 4b fix: it
//! used `PlayerType.TOP_PLAYER`, so it offered the opponent's Bench, moved
//! the Energy there, and got stuck when that Bench was empty).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Volcanionex",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("SCORCHING_STEAM"),
        // If this Pokémon is in the Active Spot, your opponent's Active Pokémon is now Burned (not when it already is).
        needs: &[Cond::IsActive(SlotExpr::This), Cond::WouldChangeConditions(OPP_ACTIVE, &[SpecialCondition::Burned])],
        steps: &[Step::new(inflict(&[SpecialCondition::Burned]))],
    }],
    attacks: &[AttackSpec {
        index: 0,
        // Heat Cyclone: move an Energy from this Pokémon to 1 of your Benched Pokémon.
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::ToBench { min: Num::Lit(1), max: Num::Lit(1), same_target: false, via_effect: false, kind: EnergyKind::Any }, to: EnergyDest::Stay, ..DiscardEnergySpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
