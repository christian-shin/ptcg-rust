//! Drayton (PRE 172): Supporter. Look at the top 7 cards of your deck. You may reveal a Pokémon and a Trainer card you
//! find there and put them into your hand. Shuffle the other cards back into your deck.
//!
//! The 7 cards are set aside (`Op::Look`); at most one Pokémon and one Trainer card may be taken (none is allowed: "You
//! may"), put into the hand by one PutIntoHand event and revealed; the others go back into the deck (PutIntoDeck) and
//! the deck is shuffled once.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);
const LOOKED_AT: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(1));
const TRAINERS: Num = Num::Min(&Num::CardCount(LOOKED_AT, Pred::Trainer), &Num::Lit(1));
const POKEMON: Num = Num::Min(&Num::CardCount(LOOKED_AT, Pred::Pokemon), &Num::Lit(1));

pub static SPEC: CardSpec = CardSpec {
    class: "Drayton",
    // Look at the top 7 cards of your deck. Choose a Pokémon and a Trainer card from those cards,
    // reveal them, and put them into your hand. Shuffle the other cards back into your deck.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: DECK, cards: CardSel::Top(Num::Lit(7)), into: 1, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec {
                from: LOOKED_AT,
                predicate: Pred::OneOf(&[Pred::Pokemon, Pred::Trainer]),
                bounds: Bounds { min: Num::Lit(0), max: Num::Add(&TRAINERS, &POKEMON) },
                into: 0,
                caps: &[Cap { kind: CapKind::Trainer, max: TRAINERS }, Cap { kind: CapKind::Pokemon, max: POKEMON }],
                distinct_types: true,
                purpose: Some(PromptPurpose::PutIntoHand),
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: LOOKED_AT, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: LOOKED_AT, cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Chosen(0), by: Who::Me, to: Who::Opp, when_empty: false })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
