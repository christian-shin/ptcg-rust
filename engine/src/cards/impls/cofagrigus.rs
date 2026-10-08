//! Cofagrigus (SSP): Law of the Underworld — put 6 damage counters on each
//! Pokémon that has an Ability (both yours and your opponent's). Spooky
//! Shot — 100.
//!
//! Twinleaf: for each of the player's Pokémon (Active then Bench), then the
//! opponent's, a CheckPokemonPowersEffect is run and, if any power is an
//! Ability, a PutCountersEffect of 60 is applied.
use crate::spec::prelude::*;

const HAS_COUNTERS: Op = Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(6), cause: CounterCause::Attack });

pub static SPEC: CardSpec = CardSpec {
    class: "Cofagrigus@Cofagrigus SSP",
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
