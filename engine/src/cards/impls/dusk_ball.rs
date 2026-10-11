//! Dusk Ball (SSP 175): look at the 7 cards from the bottom of your deck. Choose 1 Pokémon you find there, show it to your
//! opponent, and put it into your hand. Put the remaining cards back on top of your deck. Shuffle your deck afterward.
//!
//! A Look of the bottom 7 (set aside in the scratch zone 0, not yet in the hand; a Look is not a search); one Pokémon is
//! required when there is one (min 1), shown to the opponent and put into the hand (PutIntoHand); the rest go back
//! into the deck (PutIntoDeck) and the whole deck is shuffled afterward as printed, so where the rest are put back (the
//! bottom here, the text says top) can't be seen.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DuskBall",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Bottom(Num::Lit(7)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::Pokemon, bounds: Bounds { min: Num::If(&Cond::Nonempty(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::Pokemon), &Num::Lit(1), &Num::Lit(0)), max: Num::Lit(1) }, into: 1, ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Chosen(1), reveal: Some(Who::Opp), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
