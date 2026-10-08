//! Dartrix (POR / M3): Leafage — 10. Feather Shot — discard all Energy from
//! this Pokémon, and this attack does 90 damage to 1 of your opponent's
//! Pokémon.
//!
//! Twinleaf: DISCARD_ALL_ENERGY_FROM_POKEMON (CheckProvidedEnergyEffect on
//! the player's Active, then a DiscardCardsEffect of the map's cards on this
//! card's slot), then a non-cancellable ChoosePokemonPrompt and
//! DAMAGE_OPPONENT_POKEMON(90) on the choice.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dartrix@POR",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::This), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(90), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
