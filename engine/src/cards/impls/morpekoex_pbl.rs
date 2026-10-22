//! Morpeko ex (PBL 55): Wheely Draw - shuffle your hand into your deck. Then, draw 6 cards. Hangry Blaster - 40+; this attack does 40 more damage for each damage counter on this Pokémon.
//!
//! Wheely Draw is `Op::HandShuffleDraw` (Lacey PRE). Hangry Blaster adds 4 x the damage on this Pokémon (a counter is 10
//! damage: Dondozo ex DRI 66's Avenging Billow adds 1 x) before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MorpekoExPBL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::Lit(6) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::DamageOn(MY_ACTIVE), &Num::Lit(4)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
