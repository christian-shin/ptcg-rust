//! Team Rocket's Sneasel (DRI): Scratch — 20. Backstab — 20 damage to 1 of
//! your opponent's Benched Pokémon for each damage counter on it.
//!
//! Twinleaf: a PutDamageEffect of `target.damage * 2` (read when the prompt
//! resolves) on the chosen Benched Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsSneasel",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(0), target_damage_mul: 2, calc: DamageCalc::Put, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
