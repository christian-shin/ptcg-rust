//! Hoothoot (TEF): Silent Wing — 20; your opponent reveals their hand.
//!
//! Twinleaf has several `Hoothoot` classes; this port is bound to TEF. The
//! ShowCardsPrompt goes to the attacker (even for an empty hand).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Hoothoot@TEF",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Reveal(RevealSpec {
                cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)),
                to: Who::Me,
                when_empty: true,
            }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
