//! Blissey ex (TWM): Happy Switch — once during your turn, you may move a
//! Basic Energy from 1 of your Pokémon to another of your Pokémon.
//! Return — 180; you may draw until you have 6 cards in hand.
//!
//! Twinleaf: the once-per-turn marker (BLISSFUL_SWAP_MARKER) and
//! ABILITY_USED are set inside the transfer loop; Return's ConfirmPrompt
//! draws one card at a time with MOVE_CARDS.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Blisseyex",
    // Happy Switch: once during your turn, you may move a Basic Energy from 1 of your Pokémon to
    // another of your Pokémon.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("BLISSFUL_SWAP_MARKER"),
        needs: &[],
        steps: &[Step::new(Op::MoveEnergyOwn(MoveEnergyOwnSpec { to: None, energy: Pred::BasicEnergy, cancel: false, used_always: false }))],
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
