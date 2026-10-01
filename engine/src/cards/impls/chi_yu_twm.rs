//! Chi-Yu (TWM): Allure — draw 2 cards. Ground Melter — 60+; if a Stadium is
//! in play, 60 more damage, then discard that Stadium.
//!
//! Twinleaf (twilight-masquerade file): Allure is MOVE_CARDS deck → hand
//! `{ count: 2 }`; Ground Melter moves the whole stadium list to its owner's
//! discard (MOVE_CARDS without cards).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ChiYu@TWM", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            move_count_from(g, ListRef::Deck(p), ListRef::Hand(p), 2, me)?;
        }
    }
    if was_attack_used(g, e, 1, me) {
        if let Some(stadium) = g.st.stadium_card() {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 60;
            }
            if let Some(l) = g.st.locate(stadium) {
                let owner = l.owner().unwrap_or(0);
                g.run_fx(Effect::MoveCards {
                    source: l,
                    destination: ListRef::Discard(owner as u8),
                    cards: None,
                    count: None,
                    to_top: false,
                    to_bottom: false,
                    skip_cleanup: false,
                    source_card: me,
                })?;
            }
        }
    }
    Ok(())
}
