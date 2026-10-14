//! Slowpoke (PBL 29): All-You-Can-Yeet: you may discard any number of cards from your hand. Headbutt: 20.
//!
//! Any cards of the hand may be chosen (0 to all; the text says cards, not only Energy) and are discarded (one Discard event
//! over the hand, this attack its cause).
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
                purpose: Some(PromptPurpose::DiscardCard),
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
