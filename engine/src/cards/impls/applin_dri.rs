//! Applin (DRI 16): Mini Drain — 10; heal 10 damage from this Pokémon.
//!
//! Twinleaf: a HealTargetEffect on `player.active`. Two `Applin` classes
//! exist; this port is bound to DRI.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Applin@DRI", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    heal_this_pokemon(g, e, 10)
}

/// `HEAL_X_DAMAGE_FROM_THIS_POKEMON`: a HealTargetEffect on `player.active`.
pub fn heal_this_pokemon(g: &mut Game, e: EffId, damage: i32) -> R {
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    let a = g.st.players[p as usize].active;
    let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(p as usize, a) };
    g.run_fx(Effect::HealTarget { b, damage })?;
    Ok(())
}
