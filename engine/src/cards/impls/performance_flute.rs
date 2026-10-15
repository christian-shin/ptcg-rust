//! Accompanying Flute (TWM): reveal the top 5 cards of your opponent's deck. You may choose any number of Basic Pokémon you
//! find there and put those Pokémon onto their Bench. Your opponent shuffles the other cards back into their deck.
//!
//! The Look sets the 5 cards aside and they are revealed to both players (a Reveal); the chosen Basic Pokémon enter play on the opponent's Bench from there (an EnterPlay
//! caused by this Item, at most as many as there is room for), the others go back to the opponent's deck (a PutIntoDeck) and
//! the opponent's deck is shuffled. It needs room on their Bench and a card in their deck (id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AccompanyingFlute",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::BenchSpace(Who::Opp)],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(5)), into: 0, ..LookSpec::DEFAULT })),
            // "Reveal the top 5 cards": both players see them.
            Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Me, Zone::Scratch(0))), by: Who::Me, to: Who::Opp, when_empty: false })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::All(&[Pred::Pokemon, Pred::Basic]), bounds: Bounds { min: Num::Lit(0), max: Num::OpenBench(Who::Opp) }, into: 1, ..PickSpec::DEFAULT })),
            Step::new(Op::PlayFromZone(PlayFromZoneSpec { cards: 1, who: Who::Opp })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
