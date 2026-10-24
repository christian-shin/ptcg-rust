//! Floragato (JTG 17): Magical Leaf - 30+; flip a coin. If heads, this attack does 30 more damage, and heal 30 damage from this
//! Pokémon.
//!
//! The coin is flipped before the damage: heads adds 30 and flags the attack (`AttackFlagKind::OptionChosen`, as Dialga's Chrono
//! Burst does for its after-damage Energy); the heal comes after the damage, only when the flag is set.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Floragato",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::Coin(CoinSpec {
                heads: &[
                    Step::new(more_damage_if(30, Cond::True)),
                    Step::new(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::OptionChosen, value: true })),
                ],
                ..CoinSpec::DEFAULT
            })),
            Step::after_damage(Op::If(IfSpec { cond: Cond::AttackFlag(AttackFlagKind::OptionChosen), yes: &[Step::new(heal_active(30))], no: &[] })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
