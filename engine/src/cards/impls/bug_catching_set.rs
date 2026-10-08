//! Bug Catching Set (TWM): look at the top 7 cards of your deck; you may
//! reveal up to 2 [G] Pokémon / Basic [G] Energy there and put them into your
//! hand, then shuffle the other cards back into your deck.
//!
//! Twinleaf quirks kept: `max` counts matching cards in the whole deck and the
//! remaining cards are put on the bottom of the deck. Fixed in phase 4b (R4):
//! the deck is also shuffled when nothing is taken, and the card can't be played
//! with an empty deck (Rulings Compendium 779/851).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BugCatchingSet",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Deck), to: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Top(Num::Lit(7)), ..MoveSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::PokemonType(crate::types::ct::GRASS)]), Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")])]), bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::Add(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::PokemonType(crate::types::ct::GRASS)]), Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")])])), &Num::CardCount(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::PokemonType(crate::types::ct::GRASS)]), Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")])]))), &Num::Lit(2)) }, into: 1, ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(1), reveal: Some(Who::Opp), ..MoveSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::All, ..MoveSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
