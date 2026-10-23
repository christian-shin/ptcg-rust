//! Thievul (MEG 90): Greedy Hunt - 20; you may draw cards until you have 6 cards in your hand. Pitch-Black Fangs - 60.
//!
//! The draw is one Draw event (cause: this attack), asked only when it would draw something (Cynthia's Garchomp ex ASC
//! 111).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ThievulMEG",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::All(&[Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Lt, Num::Lit(6)), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)]),
            yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
