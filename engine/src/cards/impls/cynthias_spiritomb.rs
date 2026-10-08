//! Cynthia's Spiritomb (DRI): Raging Curse - 10x the damage counters on all
//! your Benched Cynthia's Pokémon; no Weakness.
//!
//! Twinleaf: a bench slot counts when any card in its `cards` list has the
//! Cynthia's tag; `effect.damage` is set to the summed slot damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasSpiritomb",
    // Raging Curse: 10 damage for each damage counter on all your Benched Cynthia's Pokémon; not
    // affected by Weakness.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
            Step::before_damage(damage_is(Num::DamageSum(SlotSel::Bench(Who::Me), SlotPred::AnyCardTag(crate::types::tag::CYNTHIAS)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
