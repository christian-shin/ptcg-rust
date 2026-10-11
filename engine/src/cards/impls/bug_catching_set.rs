//! Bug Catching Set (PRE 102): Item. Look at the top 7 cards of your deck. You may reveal up to 2 in any combination of
//! [G] Pokémon and Basic [G] Energy cards you find there and put them into your hand. Shuffle the other cards back into
//! your deck.
//!
//! The 7 cards are set aside (`Op::Look`, never a zone); the cards taken are a PutIntoHand event (revealed to the
//! opponent), the others go back to the deck (a PutIntoDeck event) and the deck is shuffled once (also when nothing was
//! taken). "You may" lets you take none. The card can't be played with an empty deck: you can't look at cards that
//! aren't there and a Trainer that can't have any effect can't be played (id840, id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BugCatchingSet",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(7)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::PokemonType(crate::types::ct::GRASS)]), Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")])]), bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::Add(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::PokemonType(crate::types::ct::GRASS)]), Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")])])), &Num::CardCount(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::PokemonType(crate::types::ct::GRASS)]), Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")])]))), &Num::Lit(2)) }, into: 1, ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Chosen(1), reveal: Some(Who::Opp), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
