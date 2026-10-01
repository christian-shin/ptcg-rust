//! Tynamo (SV11B): Hold Still — heal 10 damage from this Pokémon.
//!
//! Twinleaf: a HealTargetEffect(effect, 10) targeting the player's Active.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Tynamo@BLK", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

/// `new HealTargetEffect(effect, amount)` with `target = player.active`.
pub fn heal_own_active(g: &mut Game, e: EffId, amount: i32) -> R {
    let b = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => {
            let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
            AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }
        }
        _ => return Ok(()),
    };
    g.run_fx(Effect::HealTarget { b, damage: amount })?;
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        heal_own_active(g, e, 10)?;
    }
    Ok(())
}
