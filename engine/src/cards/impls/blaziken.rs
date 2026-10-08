//! Blaziken (DRI): Heat Blast — 70. Inferno Legs — 120; discard 2 Energy from
//! this Pokémon, and 120 damage to 1 of your opponent's Benched Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Blaziken",
    // Inferno Legs: discard 2 Energy from this Pokémon, and 120 damage to 1 of your opponent's
    // Benched Pokémon (the damage target is chosen before the Energy to discard).
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                hp: Num::Lit(120),
                target_damage_mul: 0,
                calc: DamageCalc::Auto,
                when: Cond::True,
            })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: MY_ACTIVE, selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS } })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
