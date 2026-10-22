//! Galvantula (WHT 34): Discharge — 50×; discard all [L] Energy from this Pokémon. This attack does 50 damage for each card you
//! discarded in this way.
//!
//! The Energy cards that provide [L] are discarded before the damage (a Discard event caused by the attack), and the damage is 50
//! times the cards discarded (`Num::Last`, APR B-02).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GalvantulaWHT",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Provides(ct::LIGHTNING), ..DiscardEnergySpec::DEFAULT })),
            Step::before_damage(damage_is(Num::Mul(&Num::Last, &Num::Lit(50)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
