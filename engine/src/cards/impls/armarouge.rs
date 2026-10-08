//! Armarouge (SSP): Combustion — 50. Crimson Blaster — discard all [R] Energy
//! from this Pokémon, and 180 damage to 1 of your opponent's Benched Pokémon.
//!
//! Twinleaf: a DiscardCardsEffect of every attached card named "Fire Energy"
//! on the Active (even when empty), then THIS_ATTACK_DOES_X_DAMAGE_TO_1_OF_
//! YOUR_OPPONENTS_BENCHED_POKEMON. Fixed in phase 4b (R4): the Active could
//! be chosen too (it used the "1 of your opponent's Pokémon" prefab).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Armarouge",
    // Crimson Blaster: discard all [R] Energy from this Pokémon, and 180 damage to 1 of your
    // opponent's Benched Pokémon.
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: MY_ACTIVE, selection: EnergySelection::Provides(ct::FIRE) })),
            Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                hp: Num::Lit(180),
                target_damage_mul: 0,
                calc: DamageCalc::Auto,
                when: Cond::True,
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
