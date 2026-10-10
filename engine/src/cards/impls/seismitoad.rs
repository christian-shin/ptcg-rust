//! Seismitoad (30C 84): Quaking Fist — 60; during your opponent's next turn,
//! whenever they try to use a Trainer card from their hand, they flip a
//! coin. If tails, they discard that Trainer card instead of using it.
//! Mega Punch — 180.
//!
//! Events batch 7 (user decision D6): a lasting lock on the opponent with a coin (`CoinGate::TailsDiscardsCard`) over
//! their PlayTrainer from the hand. It isn't a legality check (heads lets the play through); the play flips first, before
//! the card's own costs and before the old Stadium is discarded, and on tails the play doesn't happen: it isn't the
//! Supporter or Stadium of the turn (JP FAQ ガマゲロゲ; ミアレシティ: the same Stadium may be played again).
use crate::spec::prelude::*;

/// "Whenever they try to use a Trainer card from their hand, they flip a coin. If tails, they discard that Trainer card
/// instead of using it."
static QUAKING_FIST: LockDecl = LockDecl { error: "BLOCKED_BY_EFFECT", forbids: PLAY_TRAINER_FROM_HAND, coin: CoinGate::TailsDiscardsCard };

pub static SPEC: CardSpec = CardSpec {
    class: "Seismitoad",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::OppCannotPlay(&QUAKING_FIST) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
