//! Team Rocket's Tarountula (DRI): Take Down — 30; this Pokémon also does 10
//! damage to itself.
//!
//! Twinleaf: a DealDamageEffect(effect, 10) targeting the attacker's current
//! Active, reduced during the AttackEffect (before the main damage).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsTarountula", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let b = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => {
            let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
            AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }
        }
        _ => return Ok(()),
    };
    g.run_fx(Effect::DealDamage { b, damage: 10 })?;
    Ok(())
}
