//! Cheren (ASC 258): Supporter. Draw 3 cards.
//!
//! One Draw event of 3 cards (cause: this Supporter). It can't be played with an empty deck: a Trainer that can't have any
//! effect can't be played (id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cheren",
    // Draw 3 cards.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
