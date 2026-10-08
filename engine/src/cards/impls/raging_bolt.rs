//! Raging Bolt (SCR): Thunderburst Storm — 30 damage to 1 of the opponent's
//! Pokémon for each Energy attached to this Pokémon. Dragon Headbutt — 130.
//!
//! Twinleaf: ChoosePokemonPrompt (Active + Bench, no cancel); in the callback
//! CheckProvidedEnergyEffect(player, player.active) is summed over every
//! provided type (Special Energy providing 2 count twice), then
//! DAMAGE_OPPONENT_POKEMON (DealDamage on the Active, PutDamage on the Bench).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RagingBolt",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(30)), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
