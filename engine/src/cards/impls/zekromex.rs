//! Zekrom ex (SV11B / BLK 34): Slash — 50. Voltage Burst — 130+; 50 more
//! for each Prize card your opponent has taken; this Pokémon does 30 damage
//! to itself.
//!
//! Twinleaf: counts `6 - opponent.getPrizeLeft()` and reduces a
//! DealDamageEffect of 30 aimed at `player.active`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Zekromex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let taken = 6 - g.st.players[opp as usize].prize_left() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += 50 * taken;
        }
        let pu = p as usize;
        let target = SlotRef::new(pu, g.st.players[pu].active);
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
        g.run_fx(Effect::DealDamage { b, damage: 30 })?;
    }
    Ok(())
}
