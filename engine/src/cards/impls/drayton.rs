//! Drayton (SSP): look at the top 7 cards of your deck. Choose a Pokémon and
//! a Trainer card from those cards, reveal them, and put them into your
//! hand. Shuffle the other cards back into your deck.
//!
//! Twinleaf: throws on an empty deck; no Supporter-already-played check; the
//! card moves to the Supporter area and the play is prevented; 7 cards go to a
//! temporary list in which Energy cards are blocked, with `maxTrainers` and
//! `maxPokemons` of min(count, 1) each (different types required); the chosen
//! cards go to the hand, the rest to the bottom of the deck; ShowCards for the
//! opponent only when something was taken; a final wait-less shuffle.
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
            Step::new(Op::Move(MoveSpec { from: DECK, to: LOOKED_AT, cards: CardSel::Top(Num::Lit(7)), ..MoveSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec {
                from: LOOKED_AT,
                predicate: Pred::OneOf(&[Pred::Pokemon, Pred::Trainer]),
                bounds: Bounds { min: Num::Lit(0), max: Num::Add(&TRAINERS, &POKEMON) },
                into: 0,
                caps: &[Cap { kind: CapKind::Trainer, max: TRAINERS }, Cap { kind: CapKind::Pokemon, max: POKEMON }],
                distinct_types: true,
                msg: "CHOOSE_CARD_TO_HAND",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Move(MoveSpec { from: LOOKED_AT, to: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: LOOKED_AT, to: DECK, cards: CardSel::All, ..MoveSpec::DEFAULT })),
            Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Chosen(0), to: Who::Opp, when_empty: false })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
