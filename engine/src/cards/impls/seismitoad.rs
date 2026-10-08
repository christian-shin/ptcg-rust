//! Seismitoad (30C 84): Quaking Fist — 60; during your opponent's next turn,
//! whenever they try to use a Trainer card from their hand, they flip a
//! coin. If tails, they discard that Trainer card instead of using it.
//! Mega Punch — 180.
//!
//! Twinleaf: OPPONENT_COIN_FLIP_CANCEL_TRAINER_CARDS reduces a
//! CoinFlipCancelTrainerPlayEffect (target = the attacker's slot) setting
//! `opponent.coinFlipCancelTrainerPlayTurnsRemaining = max(.., 1)`; the flip
//! itself is `withOptionalCoinFlipCancelTrainer` in the trainer reducers.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Seismitoad",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CoinFlipCancelTrainer }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
