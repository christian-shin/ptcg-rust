//! Abra (TWM): Teleporter — once during your turn, if this Pokémon is in the
//! Active Spot, shuffle it and all attached cards into your deck. Beam — 10.
//!
//! The shuffle is `Op::RemoveFromPlay` of this Pokémon (the whole stack): one LeavePlay event with the Ability as its
//! cause, then the deck is shuffled. There is no once-per-turn marker: the card leaves play.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Abra@TWM",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::IsActive(SlotExpr::This)],
        steps: &[
            Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::This, destination: ZoneRef(Who::Me, Zone::Deck) })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
