//! Jirachi ex (30C): Wish Granter — draw cards until you have 7 cards in your
//! hand. Swift — 150; not affected by Weakness or Resistance, or by effects
//! on the opponent's Active Pokémon.
//!
//! Twinleaf: DRAW_CARDS_UNTIL_CARDS_IN_HAND is a plain `deck.moveTo(hand, n)`
//! (no MoveCardsEffect). Swift uses THIS_ATTACKS_DAMAGE_ISNT_AFFECTED_BY_EFFECTS
//! with `ignoreWeaknessAndResistance` (phase 4b; Weakness and Resistance used
//! to apply, despite the text): its own ApplyWeaknessEffect on the current
//! `effect.damage`, `effect.damage = 0`, the damage added straight to the
//! opponent's Active, then an AfterDamageEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Jirachiex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let n = 7usize.saturating_sub(g.st.players[p].hand.len());
        g.move_to(ListRef::Deck(p as u8), ListRef::Hand(p as u8), Some(n));
    }
    if was_attack_used(g, e, 1, me) {
        let dmg = match *g.e(e) {
            Effect::Attack { damage, .. } => damage,
            _ => return Ok(()),
        };
        super::mega_lopunnyex::shred_ex(g, e, dmg, true)?;
    }
    Ok(())
}
