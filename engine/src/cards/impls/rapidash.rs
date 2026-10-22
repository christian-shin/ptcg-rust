//! Rapidash (DRI 30): Ability Hurried Gait - once during your turn, you may draw a card. Fire Mane - 60.
//!
//! An Ability (once per turn) that draws 1 card; usable by the rules (D10): it can't be used with no card in the deck, as the draw would change nothing (id2231).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Rapidash",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
