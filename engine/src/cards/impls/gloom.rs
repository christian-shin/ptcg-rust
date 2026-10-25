//! Gloom (PFL 2): Disperse Drool - 20; this attack also does 20 damage to each Benched Pokémon (both yours and your opponent's).
//! (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! After the damage a Damage event of 20 on each Benched Pokémon of yours and then of your opponent's (no Weakness or Resistance,
//! `DamageCalc::Put`): the attack's own Pokémon takes the damage too when it is Benched (it isn't: the attacker is Active).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Gloom",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Bench(Who::Me), what: EachWhat::Damage(DamageCalc::Put), amount: Num::Lit(20), ..EachSlotSpec::DEFAULT })),
            Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Bench(Who::Opp), what: EachWhat::Damage(DamageCalc::Put), amount: Num::Lit(20), ..EachSlotSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
