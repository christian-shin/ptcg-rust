//! Accompanying Flute (TWM): reveal the top 5 cards of your opponent's deck,
//! and put any number of Basic Pokémon you find there onto your opponent's
//! Bench. Then, they shuffle the remaining cards back into their deck.
//!
//! Twinleaf quirks kept: the chosen cards are moved straight into the empty
//! Bench slots (MOVE_CARDS, no PlayPokemon effect) with `pokemonPlayedTurn`
//! set; with nothing chosen the opponent is shown the cards, they go back to
//! the deck, and the opponent's deck is shuffled. Neither shuffle has a
//! trailing wait.
//!
//! Fixed (phase 4b, W4): with nothing chosen Twinleaf shuffled the player's own
//! deck and left the opponent's top 5 cards at the bottom, in order.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PerformanceFlute",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::BenchSpace(Who::Opp), Cond::Nonempty(ZoneRef(Who::Opp, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Deck), to: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Top(Num::Lit(5)), ..MoveSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::All(&[Pred::Pokemon, Pred::Basic]), bounds: Bounds { min: Num::Lit(0), max: Num::OpenBench(Who::Opp) }, into: 1, ..PickSpec::DEFAULT })),
            Step::new(Op::PlayFromZone(PlayFromZoneSpec { cards: 1, who: Who::Opp })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::All, ..MoveSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
