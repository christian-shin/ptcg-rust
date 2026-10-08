//! Dusk Ball (SSP): look at the 7 cards from the bottom of your deck; choose
//! 1 Pokémon there, show it to your opponent, and put it into your hand.
//! Put the rest back and shuffle your deck.
//!
//! Twinleaf: the bottom cards are spliced into a temporary list directly (no
//! MOVE_CARDS). With nothing chosen, the `temp.cards.forEach` loop runs once
//! (its first MOVE_CARDS empties the array) if anything was looked at. With
//! a Pokémon chosen, the reveal is queued without waiting. The card is moved
//! supporter→discard (again) before the wait-less shuffle.
//!
//! Fixed (phase 4b, R3): "choose 1 Pokémon you find there" is required when
//! one of the looked-at cards is a Pokémon (min 1; it used to be min 0).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DuskBall",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Deck), to: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Bottom(Num::Lit(7)), ..MoveSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::Pokemon, bounds: Bounds { min: Num::If(&Cond::Nonempty(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::Pokemon), &Num::Lit(1), &Num::Lit(0)), max: Num::Lit(1) }, into: 1, ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(1), reveal: Some(Who::Opp), ..MoveSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::All, ..MoveSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
