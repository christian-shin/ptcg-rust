//! Chi-Yu (MEG 31): Scorching Earth — 40; if your opponent has a Stadium in
//! play, discard it. If you do, your opponent can't play any Stadium cards
//! from their hand during their next turn.
//!
//! Twinleaf: the Stadium is discarded only when it sits on the opponent's
//! side (MOVE_CARDS stadium → owner's discard); if it left play, a
//! PlayLockEffect `{ stadium: true }` follows.
//!
//! Fixed (phase 4b, R7F-4; ruling 1589): the Stadium was discarded in the
//! attack handler, before the damage; like every Stadium discard of an
//! attack it now happens in AfterAttackEffect (after the damage, before the
//! Knock Out check), and the play lock is built from a fresh AttackEffect's
//! data like the other AfterAttackEffect handlers do.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ChiYuMEGPool", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !after_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let o = match *g.e(e) {
        Effect::AfterAttack { opp, .. } => opp as usize,
        _ => return Ok(()),
    };
    let stadium = match g.st.stadium_card() {
        Some(c) => c,
        None => return Ok(()),
    };
    let owner = match g.st.players.iter().position(|pl| pl.stadium.contains(stadium)) {
        Some(i) => i,
        None => return Ok(()),
    };
    if owner != o {
        return Ok(());
    }
    move_cards(g, ListRef::Stadium(owner as u8), ListRef::Discard(owner as u8), &[stadium], me)?;
    if g.st.stadium_card() != Some(stadium) {
        return opponent_cannot_play_cards(g, e, crate::effects::play_lock::STADIUM);
    }
    Ok(())
}
