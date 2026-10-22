//! Rapidash (DRI 30): Ability Hurried Gait - once during your turn, you may draw a card. Fire Mane - 60.
//!
//! An Ability (once per turn) that draws 1 card; it can't be used with no card in the deck (id2231, as Dudunsparce TEF).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Rapidash",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
