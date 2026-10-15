//! Volcanion ex (JTG 31): Scalding Steam — once during your turn, if this Pokémon is in the Active Spot, you may make your opponent's
//! Active Pokémon Burned. Scorching Cyclone — 160; move an Energy from this Pokémon to 1 of your Benched Pokémon.
//!
//! Scalding Steam's Burn is an Ability effect: GainCondition(Burned) on the opponent's Active Pokémon with the Ability as cause (Ability
//! locks and Prevent apply); it can't be used when the Pokémon is already Burned (it would change nothing). Scorching Cyclone moves an
//! Energy onto one of your own Benched Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "VolcanionEx",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
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
