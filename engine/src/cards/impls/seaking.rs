//! Seaking (PRE): Festival Lead. Rapid Draw — 60; draw 2 cards.
//!
//! Twinleaf: the draw is MOVE_CARDS(count 2); Festival Lead is the attack's
//! runtime `barrage` flag (see Dipplin TWM).
//!
//! Fixed (phase 4b, R2): with the Ability blocked the flag was left as an
//! earlier use had set it; it is now cleared, so a blocked Seaking attacks once.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Seaking", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 2, me)?;
    if is_ability_blocked(g, p, me, None) {
        // Fixed (phase 4b, R2): a blocked Ability clears the flag set by an
        // earlier use (it used to keep it, so the attack was used twice).
        let inst = &mut g.st.cards[me as usize];
        inst.attack_barrage &= !1;
        inst.attack_barrage_shown |= 1;
    } else {
        super::dipplin_twm::festival_lead(g, p, me, false);
    }
    Ok(())
}
