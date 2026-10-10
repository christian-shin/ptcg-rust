//! Pokégear 3.0 (SVI): look at the top 7 cards of your deck; you may reveal
//! a Supporter card there and put it into your hand. Shuffle the others back.
//!
//! Twinleaf: the looked-at cards return to the bottom of the deck before the
//! reveal and the (wait-less) shuffle; the hand→supporter MOVE_CARDS is a
//! no-op (items are already in the supporter pile).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Pokegear30",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(7)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::Supporter, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, into: 1, ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Chosen(1), reveal: Some(Who::Opp), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
