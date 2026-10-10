//! Haxorus (SV11B / BLK 70): Cross-Cut — 80+; 80 more if the opponent's
//! Active is an Evolution Pokémon (not a Basic Pokémon). Axe Blast — if the
//! opponent's Active is a Basic Pokémon it is Knocked Out.
//!
//! Axe Blast is `Op::KnockOut` on the opponent's Active Pokémon: a KnockOut by an effect, asked the preventions when the
//! attack's effect runs (Mist Energy stops it, id2427) and otherwise waiting for the state check (D1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Haxorus@BLK",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(80), when: Cond::Slot(OPP_ACTIVE, SlotPred::Not(&SlotPred::Basic)) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::KnockOut(KnockOutSpec { target: OPP_ACTIVE, when: Cond::Slot(OPP_ACTIVE, SlotPred::Basic) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
