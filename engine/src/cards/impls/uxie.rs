//! Uxie (SSP 78): Painful Memories - put 2 damage counters on each of your opponent's Pokémon.
//!
//! One PlaceCounters event per Pokémon of the opponent, Active and Bench, caused by the attack (not damage: no Weakness or
//! Resistance, Bench protection like Hide 'n' Sneak applies per Pokémon).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Uxie",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), what: EachWhat::Counters, amount: Num::Lit(2), ..EachSlotSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
