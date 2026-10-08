//! Girafarig (TWM): Dual Headbutt — 30; this attack also does 10 damage to 1
//! of your Benched Pokémon.
//!
//! Twinleaf (twilight-masquerade file): no Benched Pokémon → nothing;
//! otherwise ChoosePokemonPrompt (your Bench, no cancel) and a
//! PutDamageEffect(10) on the chosen Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Girafarig@TWM",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Me), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(10), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
