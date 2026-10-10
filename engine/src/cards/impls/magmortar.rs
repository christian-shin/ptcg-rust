//! Magmortar (JTG): Magma Surge — during Pokémon Checkup, put 3 more damage counters on your opponent's Burned Pokémon.
//! Searing Flame — 90; flip a coin, if heads the opponent's Active Pokémon is now Burned.
//!
//! Magma Surge is `Modifier::CheckupDamage` (+30 on the Burn placement at Checkup, the PlaceCounters event caused by the
//! Special Condition), for each copy whose Ability works, on the opponent's Burned Pokémon only; the copies add up
//! (id2112: 2 + 3 + 3 with two Magmortar).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Magmortar",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Coin(CoinSpec { before: Cond::True, heads: &[Step::new(inflict(&[SpecialCondition::Burned]))], ..CoinSpec::DEFAULT })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::CheckupDamage(CheckupDamageSpec { amount: 30, victim: SlotPred::Condition(SpecialCondition::Burned), opponent_only: true, holder: SlotPred::Any, burn: true }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
