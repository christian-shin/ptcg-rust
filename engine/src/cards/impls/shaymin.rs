//! Shaymin (DRI): Flower Curtain — prevent all damage done to your Benched
//! Pokémon that don't have a Rule Box by attacks from your opponent's
//! Pokémon. Smash Kick — 30.
//!
//! Twinleaf: every Shaymin instance (any zone) reacts to PutDamageEffect;
//! "in play" means any Shaymin on the defending player's board; the lock probe
//! runs on the reacting instance; Rule Box is checked over all cards of the
//! target slot.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Shaymin", mask: mask(&[k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::PutDamage { b, .. } => b,
        _ => return Ok(()),
    };
    let defending = b.target.p as usize;
    if b.source.p as usize == defending {
        return Ok(());
    }
    if b.target.s == g.st.players[defending].active {
        return Ok(());
    }
    // `card instanceof Shaymin`.
    let in_play = for_each_pokemon(g, defending, PlayerType::BottomPlayer).iter().any(|(_, c, _)| g.st.cdef(*c).behavior == "Shaymin");
    if !in_play {
        return Ok(());
    }
    if g.st.slot(defending, b.target.s).cards.iter().any(|c| g.st.cdef(c).has_rule_box()) {
        return Ok(());
    }
    if is_ability_blocked(g, defending, me, None) {
        return Ok(());
    }
    g.set_prevent(e, true);
    Ok(())
}
