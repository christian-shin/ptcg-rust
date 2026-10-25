//! Hippowdon (DRI 106): Ram - 60. Super Sandstorm - 150; this attack also does 40 damage to each Benched Pokémon that has any damage
//! counters on it (both yours and your opponent's). (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! After the damage, a Damage event of 40 (no Weakness or Resistance) on each Benched Pokémon of yours and then of your opponent's
//! that has damage counters (`only_damaged`). This is a different card from Hippowdon POR 40.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HippowdonDRI",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Bench(Who::Me), what: EachWhat::Damage(DamageCalc::Put), amount: Num::Lit(40), only_damaged: true, ..EachSlotSpec::DEFAULT })),
            Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Bench(Who::Opp), what: EachWhat::Damage(DamageCalc::Put), amount: Num::Lit(40), only_damaged: true, ..EachSlotSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
