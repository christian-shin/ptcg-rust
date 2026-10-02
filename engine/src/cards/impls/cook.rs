//! Cook (FST): heal 70 damage from your Active Pokémon.
//!
//! Twinleaf reduces a HealEffect on `player.active` and lets the Trainer
//! play continue normally.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Cook", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        let a = g.st.players[p].active;
        g.run_fx(Effect::Heal { p: p as u8, target: SlotRef::new(p, a), damage: 70 })?;
    }
    Ok(())
}
