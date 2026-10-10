//! Team Rocket's Sneasel (DRI): Scratch — 20. Strike the Sleeper — 20 damage to
//! 1 of your opponent's Benched Pokémon for each damage counter on it.
//!
//! A Damage event of twice the chosen Pokémon's damage (20 per counter), read when the choice resolves (no Weakness or
//! Resistance, APR B-08).
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
