//! Maractus (JTG): Explosive Needle — if this Pokémon is in the Active Spot
//! and is Knocked Out by damage from an attack from your opponent's Pokémon,
//! put 6 damage counters on the Attacking Pokémon. Corner — 20; the
//! Defending Pokémon can't retreat during your opponent's next turn.
//!
//! Twinleaf quirks kept: Explosive Needle runs on any PutDamageEffect whose
//! target's top card is this Maractus during the attack phase (Active or
//! Bench, either player's attack); if the pending damage is at least the
//! remaining HP it adds 60 straight to the source's `damage`, before the
//! damage itself is applied (and even if it is later prevented or reduced).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Maractus@JTG", mask: mask(&[k::PUT_DAMAGE, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PutDamage { b, damage, .. } = *g.e(e) {
        let t = b.target;
        let (tp, ts) = (t.p as usize, t.s);
        if !g.st.slot(tp, ts).cards.contains(me) {
            return Ok(());
        }
        if g.st.slot_pokemon(tp, ts) != Some(me) || g.st.phase != GamePhase::Attack || is_ability_blocked(g, tp, me, None) {
            return Ok(());
        }
        let hp = crate::engine::check::check_hp(g, tp, ts)?;
        let current = hp - g.st.slot(tp, ts).damage;
        if damage >= current {
            let s = b.source;
            g.st.players[s.p as usize].slots[s.s as usize].damage += 60;
        }
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        return block_retreat(g, e);
    }
    Ok(())
}
