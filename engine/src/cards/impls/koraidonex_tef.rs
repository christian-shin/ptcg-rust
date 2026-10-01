//! Koraidon ex (TEF): Retribution Strike — 20+, 10 more for each damage
//! counter on this Pokémon. Kaiser Tackle — 280; this Pokémon does 60 damage
//! to itself.
//!
//! Twinleaf (temporal-forces file): Retribution Strike adds
//! `player.active.damage` (the Active, not necessarily this Pokémon);
//! Kaiser Tackle reduces a DealDamageEffect(60) targeting `player.active`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Koraidonex@TEF", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let d = g.st.slot(p, g.st.players[p].active).damage;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += d;
        }
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let target = SlotRef::new(p as usize, g.st.players[p as usize].active);
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
        g.run_fx(Effect::DealDamage { b, damage: 60 })?;
    }
    Ok(())
}
