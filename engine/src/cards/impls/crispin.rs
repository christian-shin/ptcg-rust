//! Crispin (SCR): search your deck for up to 2 Basic Energy cards of
//! different types, reveal them, put 1 into your hand and attach the other to
//! 1 of your Pokémon, then shuffle.
//!
//! Fixed (phase 4b): the search prompt is `differentTypes` (a duplicate type
//! can't be picked any more, so the old CAN_ONLY_SELECT_TWO_DIFFERENT_ENERGY_TYPES
//! throw in its callback is gone).
//!
//! Fixed (R1-17, rulings 779 and 851): it can't be played with an empty deck
//! (CANNOT_PLAY_THIS_CARD, before the card moves).
//!
//! Twinleaf order kept: the ShowCards, AttachEnergy and ShuffleDeck prompts
//! are all created by the search callback (the shuffle before the attach is
//! answered); the final shuffle has no trailing WaitPrompt; the ShowCards
//! prompt is created even when nothing was selected.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);
const HELD: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(1));

pub static SPEC: CardSpec = CardSpec {
    class: "Crispin",
    // Search your deck for up to 2 Basic Energy cards of different types, reveal them, put 1 into
    // your hand and attach the other to 1 of your Pokémon, then shuffle.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec {
                from: DECK,
                predicate: Pred::BasicEnergy,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                into: 0,
                distinct_types: true,
                msg: "CHOOSE_CARD_TO_HAND",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Chosen(0), to: Who::Opp, when_empty: true })),
            Step::new(Op::Move(MoveSpec { from: DECK, to: HELD, cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Cmp(Num::RegCount(1), CmpOp::Eq, Num::Lit(2)),
                yes: &[Step::new(Op::Attach(AttachSpec {
                    from: HELD,
                    predicate: Pred::BasicEnergy,
                    slots: AttachSlots::BenchActive,
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    different_targets: true,
                    ..AttachSpec::DEFAULT
                }))],
                no: &[],
            })),
            Step::new(Op::Move(MoveSpec { from: HELD, to: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::All, ..MoveSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
