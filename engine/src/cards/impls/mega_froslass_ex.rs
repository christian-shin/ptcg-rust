//! Mega Froslass ex (M2a): Resentful Refrain — 50x per card in your
//! opponent's hand (`effect.damage = 50 * handCount`). Absolute Snow — 150;
//! your opponent's Active Pokémon is now Asleep, via
//! YOUR_OPPPONENTS_ACTIVE_POKEMON_IS_NOW_ASLEEP (an AddSpecialConditionsEffect
//! in the attack handler; phase 4b: it used to be AFTER_ATTACK with an
//! AddSpecialConditionsPowerEffect, which Mist Energy etc. could not prevent).
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
                Step::before_damage(inflict(&[SpecialCondition::Asleep], Cause::Attack)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
