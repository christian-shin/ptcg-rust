//! Mega Froslass ex (M2a / ASC 47): Resentful Refrain - 50x for each card in your opponent's hand. Absolute Snow - 150; your
//! opponent's Active Pokémon is now Asleep.
//!
//! Resentful Refrain sets the damage to 50 times the cards in the opponent's hand, before the damage. Asleep is
//! an effect of the attack: a GainCondition event on the opponent's Active Pokémon with the attack as its cause (Mist Energy and
//! the like prevent it).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaFroslassEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), &Num::Lit(50)), when: Cond::True })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(inflict(&[SpecialCondition::Asleep])),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
