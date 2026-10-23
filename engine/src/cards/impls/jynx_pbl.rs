//! Jynx (PBL 32): Intense Kiss — at the end of your opponent's next turn, discard the Defending Pokémon and all attached cards.
//! Psy Bolt — 50; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed.
//!
//! Intense Kiss is an effect on the Defending Pokémon: moving it to the Bench removes it, and neither the old nor the new Active
//! Pokémon is discarded (id2404). At the end of the opponent's next turn (APR E-21, whose example is the same text on Team
//! Rocket's Grimer) the Pokémon and every card attached to it leave play for its owner's discard pile: a LeavePlay, not a Knock
//! Out, so no Prize cards. Psy Bolt's Paralysis on heads is a GainCondition by the attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "JynxPBL",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::AtEndOfOpponentsNextTurn(&DISCARD_THE_DEFENDING_POKEMON) }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
