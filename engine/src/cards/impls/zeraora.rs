//! Zeraora (DRI): Scratch — 20. Thunder Blitz — discard all Energy from this
//! Pokémon; 210 damage to 1 of the opponent's Benched Pokémon ex.
//!
//! The "any ex" test counts only the opponent's Benched Pokémon (phase 4b fix:
//! it used forEachPokemon without a slot check, so an ex in the Active Spot
//! alone let the attack through to a prompt with every Benched target blocked,
//! which was unanswerable). Phase 4b R7E (ruling 1790): with no Benched ex the
//! attack is still usable (Twinleaf threw CANNOT_PLAY_THIS_CARD): the Energy is
//! discarded and there is no damage and no prompt.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Zeraora@Zeraora DRI",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: MY_ACTIVE, selection: EnergySelection::AllProvided })),
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Opp), SlotPred::Top(Pred::Tag(tag::POKEMON_EX_LOWER))), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(210), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
