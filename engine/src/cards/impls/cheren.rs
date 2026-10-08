//! Cheren (EPO / ASC): draw 3 cards.
//!
//! Twinleaf throws CANNOT_PLAY_THIS_CARD when the deck is empty.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cheren",
    // Draw 3 cards.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
