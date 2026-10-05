//! Cook (FST): heal 70 damage from your Active Pokémon.
//!
//! Twinleaf reduces a HealEffect on `player.active` and lets the Trainer
//! play continue normally.
//! Fixed (phase 4b, R3): can't be played while the Active has no damage (a
//! Trainer with no possible effect is unplayable; Rulings Compendium 851).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Cook", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        let a = g.st.players[p].active;
        if g.st.slot(p, a).damage <= 0 {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        g.run_fx(Effect::Heal { p: p as u8, target: SlotRef::new(p, a), damage: 70 })?;
    }
    Ok(())
}
