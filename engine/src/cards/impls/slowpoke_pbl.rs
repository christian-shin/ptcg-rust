//! Slowpoke (PBL, Slowpoke M5): All-You-Can-Yeet — you may discard any number
//! of cards from your hand. Headbutt — 20.
//!
//! Twinleaf (pitch-black file): empty hand → nothing; ChooseCardsPrompt on
//! the hand (superType ANY), min 0, max hand size, no cancel; the chosen
//! cards go to the discard in one MOVE_CARDS.
//!
//! Twinleaf quirk kept: `matchesPromptFilter` compares `card.superType ===
//! SuperType.ANY` literally, so no card matches and only the empty
//! selection is possible — the attack never discards anything.
//!
//! Spec: the Twinleaf quirk is kept: no card of the hand is ever eligible, so nothing is asked and nothing is discarded.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Slowpoke@PBL",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Pick(PickSpec {
            from: ZoneRef(Who::Me, Zone::Hand),
            predicate: Pred::False,
            bounds: Bounds { min: Num::Lit(0), max: Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)) },
            into: 0,
            msg: "CHOOSE_CARD_TO_DISCARD",
            ..PickSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
