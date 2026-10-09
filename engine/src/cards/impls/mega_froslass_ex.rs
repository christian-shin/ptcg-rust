//! Mega Froslass ex (M2a): Resentful Refrain — 50x per card in your
//! opponent's hand (`effect.damage = 50 * handCount`). Absolute Snow — 150;
//! your opponent's Active Pokémon is now Asleep, via
//! Absolute Snow's Sleep is an effect of the attack: a GainCondition(Asleep) on
//! the opponent's Active with the attack's cause (Mist Energy etc. prevent it).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaFroslassex",
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
