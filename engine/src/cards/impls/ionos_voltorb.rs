//! Iono's Voltorb (JTG 47): Voltaic Chain — 20+; this attack does 20 more damage for each [L] Energy attached to all of your
//! Iono's Pokémon.
//!
//! The count is the [L] Energy the Iono's Pokémon (the Iono's tag, Active and Benched) provide, as any Energy that provides [L]
//! counts, not only Basic [L] Energy (id2119; "Energy attached" is the Energy provided, id2237), before the damage.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "IonosVoltorb",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec {
            op: DamageOp::Add,
            hp: Num::Mul(&Num::EnergyOn(SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Tag(tag::IONOS)), EnergyUnit::Provided(ct::LIGHTNING)), &Num::Lit(20)),
            when: Cond::True,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
