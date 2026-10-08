//! Magmortar (JTG): Magma Surge — during Pokémon Checkup, put 3 more damage
//! counters on your opponent's Burned Pokémon. Searing Flame — 90; flip a
//! coin, if heads the opponent's Active Pokémon is now Burned.
//!
//! Twinleaf: on each BetweenTurnsEffect the owner is found by scanning
//! [player, opponent] (last match wins), a generic ability probe is run for
//! the owner, and `burnDamage += 30` when the ending player is the owner's
//! opponent and its Active is Burned.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Magmortar",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Coin(CoinSpec { before: Cond::True, heads: &[Step::new(inflict(&[SpecialCondition::Burned], Cause::Attack))], ..CoinSpec::DEFAULT })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::CheckupDamage(CheckupDamageSpec { amount: 30, victim: SlotPred::Condition(SpecialCondition::Burned), opponent_only: true, holder: SlotPred::Any, burn: true }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
