//! Bramblin (PFL 46): Sneaky Placement — place 1 damage counter on 1 of your opponent's Pokémon.
//!
//! One PlaceCounters event on the chosen Pokémon (the opponent's Active or Benched Pokémon, chosen at step D) caused by the
//! attack: counters, not damage (APR C-07), so Weakness and Resistance never apply.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Bramblin",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::PlaceCounters(PlaceCountersSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }),
            counters: Num::Lit(1),
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
