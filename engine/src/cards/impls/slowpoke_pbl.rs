//! Slowpoke (PBL, Slowpoke M5): All-You-Can-Yeet — you may discard any number
//! of cards from your hand. Headbutt — 20.
//!
//! Spec: any card of the hand may be chosen (the printed text says any number
//! of cards, not only Energy), min 0; the chosen cards are discarded.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Slowpoke@PBL",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: Pred::Any,
                bounds: Bounds { min: Num::Lit(0), max: Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)) },
                into: 0,
                msg: "CHOOSE_CARD_TO_DISCARD",
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
