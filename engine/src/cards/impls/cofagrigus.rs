//! Cofagrigus (SSP): Law of the Underworld — put 6 damage counters on each Pokémon that has an Ability (both yours and
//! your opponent's). Spooky Shot — 100.
//!
//! Law of the Underworld is two ForEach loops (yours, then the opponent's) of `Op::PlaceCounters`: one PlaceCounters
//! event per Pokémon, caused by the attack. Counters are not damage (APR C-07: no Weakness, Resistance or damage
//! modifiers), and "prevent all effects of attacks" (Hide 'n' Sneak, Mist Energy) refuses the placement on that Pokémon
//! without stopping the others.
use crate::spec::prelude::*;

const HAS_COUNTERS: Op = Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(6) });

pub static SPEC: CardSpec = CardSpec {
    class: "Cofagrigus@SSP",
    // Law of the Underworld: put 6 damage counters on each Pokémon that has an Ability (yours
    // first, then your opponent's).
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::ForEach(ForEachSpec { over: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::HasAbility), body: &[Step::new(HAS_COUNTERS)] })),
            Step::after_damage(Op::ForEach(ForEachSpec { over: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::HasAbility), body: &[Step::new(HAS_COUNTERS)] })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
