//! Gwynn (PBL 78 / M5): discard up to 2 Pokémon that don't have a Rule Box from your hand, and draw 3 cards for each card you
//! discarded in this way.
//!
//! Pick then Discard (from the hand, D1: Discard is hand and deck cards); the Draw counts the cards chosen. Played from the hand
//! it takes at least 1 when able (id2301: a Trainer's "up to"); used through Look-Alike Show it may take 0 (D5, APR D-05). A
//! draw with an empty deck still discards (id2419: do as much as you can).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Gwynn",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::RuleBox)]), bounds: Bounds { min: Num::If(&Cond::TrainerViaAttack, &Num::Lit(0), &Num::Lit(1)), max: Num::Lit(2) }, into: 0, ..PickSpec::DEFAULT })),
            Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Mul(&Num::RegCount(0), &Num::Lit(3))) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
