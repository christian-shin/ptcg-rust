//! Blissey ex (TWM 134): Happy Switch - once during your turn, you may move a Basic Energy from 1 of your Pokémon to
//! another of your Pokémon. Return - 180; you may draw cards until you have 6 cards in your hand.
//!
//! Happy Switch is a MoveEnergy between your own Pokémon (APR C-10), once per turn (the usage record of the Ability).
//! Return's draw is one Draw event of the cards missing from 6, after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Blisseyex",
    // Happy Switch: once during your turn, you may move a Basic Energy from 1 of your Pokémon to
    // another of your Pokémon.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[Step::new(Op::MoveEnergyOwn(MoveEnergyOwnSpec { to: None, energy: Pred::BasicEnergy, cancel: false }))],
    }],
    // Return: you may draw until you have 6 cards in hand.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::True,
            msg: "WANT_TO_DRAW_UNTIL_6",
            yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
