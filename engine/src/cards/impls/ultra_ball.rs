//! Ultra Ball (SVI 196): you can use this card only if you discard 2 other cards from your hand. Search your deck for a
//! Pokémon, reveal it, and put it into your hand. Then, shuffle your deck.
//!
//! The discard is a cost (`Step::cost`, APR E-30; id1188: not usable with an empty deck, no discard for no effect), a Discard
//! event over the hand; the chosen Pokémon is revealed before it moves (id1131) and goes into the hand (PutIntoHand); the deck
//! is shuffled after the search (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "UltraBall",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(2), max: Num::Lit(2) }, into: 0, purpose: Some(PromptPurpose::DiscardCard), ..PickSpec::DEFAULT })),
            Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, into: 1, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
