//! Galarian Zigzagoon (FST 159, support card): Lick — 10; flip a coin, if
//! heads your opponent's Active Pokémon is now Paralyzed.
//!
//! Twinleaf: after the attack (AfterAttackEffect) a COIN_FLIP_PROMPT; on
//! heads ADD_PARALYZED_TO_PLAYER_ACTIVE for the opponent, i.e. an
//! AddSpecialConditionsPowerEffect (not the attack effect).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "GalarianZigzagoon@Galarian Zigzagoon FST 159",
    attacks: &[AttackSpec {
        index: 0,
        // The Paralysis is an Ability-style effect (AddSpecialConditionsPowerEffect), as today.
        steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
