//! Meloetta (PFL 40): Soothing Melody - heal 120 damage from 1 of your Benched [P] Pokémon. Magical Shot - 50.
//!
//! The Benched Pokémon is chosen at step D among those of your Bench that are [P] now (APR D-07, the type it has now); the heal is a
//! RemoveCounters event (Spritzee POR's form).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Meloetta",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Heal(HealSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::TypeIs(ct::PSYCHIC)) }),
            hp: Num::Lit(120),
            clear_conditions: false,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
