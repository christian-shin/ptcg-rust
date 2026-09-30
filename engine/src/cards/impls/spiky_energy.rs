//! Spiky Energy (JTG): provides [C]. If the Pokémon this card is attached to
//! is in the Active Spot and is damaged by an attack from your opponent's
//! Pokémon (even if it is Knocked Out), put 2 damage counters on the
//! Attacking Pokémon.
//!
//! Twinleaf: reacts to DealDamageEffect (before any damage is put, whatever
//! its amount) during the attack phase; the block check is made for the
//! attacking player; the counters are a PutCountersEffect on the attacker.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SpikyEnergy", mask: mask(&[k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::DealDamage { b, .. } => b,
        _ => return Ok(()),
    };
    let t = b.target;
    if !g.st.slot(t.p as usize, t.s).cards.contains(me) || g.st.phase != GamePhase::Attack {
        return Ok(());
    }
    if t.p == b.player || g.st.players[t.p as usize].active != t.s {
        return Ok(());
    }
    if is_special_energy_blocked(g, b.player as usize, me, t, false) {
        return Ok(());
    }
    let mut pb = b;
    pb.target = b.source;
    g.run_fx(Effect::PutCounters { b: pb, damage: 20 })?;
    Ok(())
}
