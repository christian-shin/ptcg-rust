//! Deoxys (CRI 34): Psyspeed - 30; you may draw cards until you have 5 cards in your hand.
//!
//! The draw is one Draw event (cause: this attack), asked only when it would draw something (5 or more cards in hand, or an
//! empty deck, ask nothing), like Cynthia's Garchomp ex's Corkscrew Dive.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DeoxysCRI34",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                // Nothing to ask with 5 or more cards in hand or an empty deck.
                when: Cond::All(&[
                    Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Lt, Num::Lit(5)),
                    Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                ]),
                yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(5)) }))],
                no: &[],
            })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
