//! Crispin (SCR 133): search your deck for up to 2 Basic Energy cards of different types, reveal them, and put 1 of them
//! into your hand. Attach the other to 1 of your Pokémon. Then, shuffle your deck.
//!
//! The search needs a deck with a card in it (id840). The chosen cards (at most one per type) are revealed to the opponent,
//! then set aside (a Look into the scratch zone 1, not yet in any hand). With two cards the player chooses which one is
//! attached (an Attach event onto one of their Pokémon, the Supporter its cause) and the other goes into the hand
//! (PutIntoHand); with one card it goes into the hand. The text puts the shuffle last; here it comes right after the
//! set-aside (the deck holds the same cards either way, so only the prompt order differs; noted by the B73 sweep).
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
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Chosen(0), by: Who::Me, to: Who::Opp, when_empty: true })),
            Step::new(Op::Look(LookSpec { from: DECK, cards: CardSel::Chosen(0), into: 1, ..LookSpec::DEFAULT })),
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
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: HELD, cards: CardSel::All, ..PutIntoHandSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
