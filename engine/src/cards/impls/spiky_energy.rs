//! Spiky Energy (JTG): provides [C]. If the Pokémon this card is attached to
//! is in the Active Spot and is damaged by an attack from your opponent's
//! Pokémon (even if it is Knocked Out), put 2 damage counters on the
//! Attacking Pokémon.
//!
//! Twinleaf: the block check is made for the attacking player; the counters
//! are a PutCountersEffect on the attacker.
//!
//! Fixed (phase 4b, R7F-15): it reacted to DealDamageEffect (before any damage
//! is put, whatever its amount), so it also fired when the damage was
//! prevented (Crustle's Mysterious Rock Inn, ...) or reduced to 0 although the
//! Pokémon was not damaged. It now reacts to AfterDamageEffect, like Punk
//! Helmet and Lucky Helmet (text: "is damaged by an attack"; rulings 1646,
//! 1839: it stacks and works wherever the Pokémon end up).
//!
//! R7 merge (attack flow chart step 7, after the step 6 effects of the attack): when the attack's after-damage
//! window is open the counters are queued behind the attack's own Energy discards and are placed only if the
//! card is still attached (`AfterDmgStep::Retaliate`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SpikyEnergy", mask: mask(&[k::AFTER_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let b = match *g.e(e) {
        Effect::AfterDamage { b, .. } => b,
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
    // Step 7 of the attack flow chart: the effects on the Defending Pokémon come after the effects of the
    // attack's own text, so an attack that discards this card (Duraludon's Hyper Beam) stops it.
    if g.after_damage_open(b.attack_effect) {
        g.push_after_damage(b.attack_effect, crate::game::AfterDmgStep::Retaliate { card: me, from: t, b: pb, damage: 20 });
        return Ok(());
    }
    g.run_fx(Effect::PutCounters { b: pb, damage: 20 })?;
    Ok(())
}
