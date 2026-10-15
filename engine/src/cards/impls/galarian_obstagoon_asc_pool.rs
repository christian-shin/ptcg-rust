//! Galarian Obstagoon (ASC 132): Scarring Shout - 70 damage for each damage counter on your opponent's Active Pokémon.
//! Punk Smash - 160; discard an Energy from this Pokémon.
//!
//! Scarring Shout sets the damage (70 x counters, read before the damage). Punk Smash discards one Energy after the
//! damage: a LeavePlay (D1), chosen by the player, cause: this attack.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "GalarianObstagoon",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::DamageOn(SlotExpr::Active(Who::Opp)), &Num::Lit(7)))),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::Prompt { ty: None, min: 1, max: 1, into: None }, ..DiscardEnergySpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
