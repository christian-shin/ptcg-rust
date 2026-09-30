//! Lucky Helmet (TWM, tool): if the Pokémon this card is attached to is in
//! the Active Spot and is damaged by an attack from your opponent's Pokémon
//! (even if it is Knocked Out), draw 2 cards.
//!
//! Twinleaf: reacts to AfterDamageEffect on the holder; the tool block probe
//! is a bare ToolEffect for the attacking player and runs first; the draw is
//! MOVE_CARDS(count 2) from the attacked player's deck (no phase check).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LuckyHelmet", mask: mask(&[k::AFTER_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (b, damage) = match *g.e(e) {
        Effect::AfterDamage { b, damage } => (b, damage),
        _ => return Ok(()),
    };
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
        return Ok(());
    }
    if g.run_fx(Effect::Tool { p: b.player, card: me }).is_err() {
        return Ok(());
    }
    if damage <= 0 || b.player == t.p || g.st.players[t.p as usize].active != t.s {
        return Ok(());
    }
    let o = 1 - b.player as usize;
    move_count_from(g, ListRef::Deck(o as u8), ListRef::Hand(o as u8), 2, me)
}
