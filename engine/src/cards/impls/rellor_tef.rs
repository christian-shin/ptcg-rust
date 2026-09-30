//! Rellor (TEF): Slight Intrusion — 30; this Pokémon also does 10 damage to
//! itself.
//!
//! Twinleaf: the self-damage is a DealDamageEffect on the player's Active,
//! reduced during the AttackEffect (before the attack's own damage).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Rellor@Rellor TEF", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
    g.run_fx(Effect::DealDamage { b: AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target }, damage: 10 })?;
    Ok(())
}
