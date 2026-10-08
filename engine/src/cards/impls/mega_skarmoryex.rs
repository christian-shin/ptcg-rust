//! Mega Skarmory ex (POR / M3): Sonic Ripper — shuffle all Energy from this
//! Pokémon into your deck; 220 damage to 1 of your opponent's Pokémon (no
//! Weakness/Resistance for the Bench).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaSkarmoryex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: MY_ACTIVE, selection: EnergySelection::AllIntoDeck })),
            Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                hp: Num::Lit(220),
                target_damage_mul: 0,
                calc: DamageCalc::Auto,
                when: Cond::True,
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
