//! Haxorus (SV11B / BLK 70): Cross-Cut — 80+; 80 more if the opponent's
//! Active is not a Basic Pokémon. Axe Bomber — if the opponent's Active is a
//! Basic Pokémon it is Knocked Out (Mist-blockable KnockOutOpponentEffect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Haxorus@Haxorus SV11B",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(80), when: Cond::Slot(OPP_ACTIVE, SlotPred::Not(&SlotPred::Top(Pred::Basic))) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::KnockOut(KnockOutSpec { target: OPP_ACTIVE, mode: KnockOutMode::Opponent, when: Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::Basic)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
