//! Galarian Obstagoon (ASC 132): Scarring Shout — 70x damage counters on the
//! opponent's Active. Punk Smash — 160; discard an Energy from this Pokémon.
//!
//! Twinleaf: `damage = 70 * floor(opponent.active.damage / 10)`;
//! DISCARD_UP_TO_X_ENERGY_FROM_THIS_POKEMON(1, {}, 1).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "GalarianObstagoonASCPool",
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
