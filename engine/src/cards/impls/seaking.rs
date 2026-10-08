//! Seaking (PRE): Festival Lead. Rapid Draw — 60; draw 2 cards.
//!
//! Twinleaf: the draw is MOVE_CARDS(count 2); Festival Lead is the attack's
//! runtime `barrage` flag (see Dipplin TWM).
//!
//! Fixed (phase 4b, R2): with the Ability blocked the flag was left as an
//! earlier use had set it; it is now cleared, so a blocked Seaking attacks once.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Seaking",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            // Festival Lead: with Festival Grounds in play (and the Ability working) this attack may be used twice.
            Step::before_damage(Op::If(IfSpec {
                cond: Cond::All(&[Cond::StadiumInPlay(Pred::Name("Festival Grounds")), Cond::Not(&Cond::AbilityBlocked)]),
                yes: &[Step::new(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::Barrage, value: true }))],
                no: &[Step::new(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::Barrage, value: false }))],
            })),
            Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
