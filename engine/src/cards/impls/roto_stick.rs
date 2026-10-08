//! Roto-Stick (PRE, Item): look at the top 4 cards of your deck; reveal any
//! number of Supporters there and put them into your hand; shuffle the rest
//! back into your deck.
//!
//! Twinleaf: the chosen cards are moved, the rest put back, then (if any
//! were chosen) a ShowCards prompt is awaited before the final shuffle,
//! which has no animation wait.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RotoStick",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Deck), to: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Top(Num::Lit(4)), ..MoveSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::Supporter, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(4) }, into: 1, ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(1), reveal: Some(Who::Opp), ..MoveSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::All, ..MoveSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
