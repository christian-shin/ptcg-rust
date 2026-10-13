//! Cynthia's Spiritomb (DRI): Raging Curse — 10× the damage counters on all your Benched Cynthia's Pokémon; no Weakness.
//!
//! The attack sets the flag NoWeakness and its damage to the damage on your Benched Cynthia's Pokémon (the Pokémon card's tag:
//! a Pokémon whose attached card carries the tag isn't one).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasSpiritomb",
    // Raging Curse: 10 damage for each damage counter on all your Benched Cynthia's Pokémon; not
    // affected by Weakness.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
            Step::before_damage(damage_is(Num::DamageSum(SlotSel::Bench(Who::Me), SlotPred::Tag(crate::types::tag::CYNTHIAS)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
