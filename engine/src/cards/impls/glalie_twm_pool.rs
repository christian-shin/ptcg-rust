//! Glalie (TWM 52): Damage Beat — 20 damage for each damage counter on your
//! opponent's Active Pokémon. Crazy Headbutt — 140; discard an Energy from
//! this Pokémon.
//!
//! Twinleaf: Crazy Headbutt is DISCARD_UP_TO_X_ENERGY_FROM_THIS_POKEMON(1, {}, 1).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "GlalieTWMPool",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::DamageOn(SlotExpr::Active(Who::Opp)), &Num::Lit(2)))),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotExpr::Active(Who::Me), selection: EnergySelection::Prompted { max: 1, min: 1 } })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
