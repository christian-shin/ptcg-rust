//! Seismitoad (30C 84): Quaking Fist — 60; during your opponent's next turn,
//! whenever they try to use a Trainer card from their hand, they flip a
//! coin. If tails, they discard that Trainer card instead of using it.
//! Mega Punch — 180.
//!
//! Twinleaf: OPPONENT_COIN_FLIP_CANCEL_TRAINER_CARDS reduces a
//! CoinFlipCancelTrainerPlayEffect (target = the attacker's slot) setting
//! `opponent.coinFlipCancelTrainerPlayTurnsRemaining = max(.., 1)`; the flip
//! itself is `withOptionalCoinFlipCancelTrainer` in the trainer reducers.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Seismitoad", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !after_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let e = real_attack(g, e);
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: source };
    g.run_fx(Effect::CoinFlipCancelTrainerPlay { b })?;
    Ok(())
}
