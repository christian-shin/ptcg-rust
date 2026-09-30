//! Hoothoot (TEF): Silent Wing — 20; your opponent reveals their hand.
//!
//! Twinleaf has several `Hoothoot` classes; this port is bound to TEF. The
//! ShowCardsPrompt goes to the attacker (even for an empty hand).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Hoothoot@TEF", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let id = g.player_id(p);
        g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
    }
    Ok(())
}
