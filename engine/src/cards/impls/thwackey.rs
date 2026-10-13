//! Thwackey (TWM 15): Boom Boom Groove - once during your turn, if your Active Pokémon has the Festival Lead Ability, you
//! may search your deck for a card and put it into your hand. Then, shuffle your deck. Beat - 50.
//!
//! The search is for any card, so at least 1 is chosen (id2301); it is put into the hand by a PutIntoHand event (cause: this
//! Ability, not revealed), then the deck is shuffled once (APR E-19). Once per turn is the usage marker. "Has the
//! Festival Lead Ability" is the Ability after effects (`SlotPred::HasAbilityNamed`, a checked read; JP FAQ Thwackey /
//! Flutter Mane: with Midnight Fluttering turning Dipplin's Festival Lead off, Boom Boom Groove can't be used; batch 8
//! C12c: it read the printed Ability).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Thwackey",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("BOOM_BOOM_DRUM_MARKER"),
        // If your Active Pokémon has the Festival Lead Ability.
        needs: &[Cond::Slot(MY_ACTIVE, SlotPred::HasAbilityNamed("Festival Lead"))],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
