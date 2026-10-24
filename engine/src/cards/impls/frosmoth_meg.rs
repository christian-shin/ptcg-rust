//! Frosmoth (MEG 43): Chilling Wings - this attack does 20 damage to each of your opponent's Pokémon. (Don't apply Weakness and
//! Resistance for Benched Pokémon.) Your opponent's Active Pokémon is now Asleep.
//!
//! A Damage event of 20 on each of the opponent's Pokémon (Weakness and Resistance for the Active Pokémon only; Team Rocket's Arbok
//! DRI 113's form), then Asleep on the Defending Pokémon. This is a different card from Frosmoth ASC 53.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FrosmothMEG",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), what: EachWhat::Damage(DamageCalc::Auto), amount: Num::Lit(20), ..EachSlotSpec::DEFAULT })),
            Step::after_damage(inflict(&[SpecialCondition::Asleep])),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
