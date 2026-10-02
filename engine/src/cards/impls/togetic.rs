//! Togetic (SSP / ASC): Drain Kiss - 30; heal 30 damage from this Pokémon.
//!
//! Twinleaf: `new HealEffect(player, player.active, 30)` (a HealEffect, not
//! the attack-side HealTargetEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Togetic@SSP|ASC", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, a) = match *g.e(e) {
            Effect::Attack { p, .. } => (p, g.st.players[p as usize].active),
            _ => return Ok(()),
        };
        g.run_fx(Effect::Heal { p, target: SlotRef::new(p as usize, a), damage: 30 })?;
    }
    Ok(())
}
