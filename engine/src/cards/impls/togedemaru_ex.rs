//! Togedemaru ex (ASC 149): Stun Needle - 20; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed. Spiky Rolling -
//! 80+; if this Pokémon used Spiky Rolling during your last turn, this attack does 80 more damage.
//!
//! Stun Needle: Litten TEF's form. Spiky Rolling reads the attack the player used during their previous turn and the Pokémon that used
//! it (`Cond::AttackedLastTurn` with `AttackUser::This`): this Pokémon, even if it retreated and came back; a failed attack or a
//! turn without that attack leaves no record.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TogedemaruEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))] },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(more_damage_if(80, Cond::AttackedLastTurn { who: Who::Me, by: Pred::Any, user: AttackUser::This, attack: AttackPred::Named("Spiky Rolling") }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
