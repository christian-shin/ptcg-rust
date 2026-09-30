//! Crustle (DRI): Mysterious Stone House — prevent all damage done to this
//! Pokémon by attacks from your opponent's Pokémon ex. Great Scissors — 120;
//! this attack's damage isn't affected by any effects on your opponent's
//! Active Pokémon.
//!
//! Twinleaf: the ability only reacts to PutDamageEffect while this card is
//! the target's top Pokémon, during the attack phase; the lock check reduces
//! a real PowerEffect for Mysterious Stone House. Great Scissors applies
//! Weakness/Resistance on its own ApplyWeaknessEffect (ignore flags unset),
//! zeroes the attack damage, adds the damage straight to the opponent's
//! Active and reduces an AfterDamageEffect (no PutDamageEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Crustle@DRI", mask: mask(&[k::PUT_DAMAGE, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PutDamage { b, .. } = *g.e(e) {
        let t = b.target;
        if !g.st.slot(t.p as usize, t.s).cards.contains(me) {
            return Ok(());
        }
        let pokemon = g.st.slot_pokemon(t.p as usize, t.s);
        let source_card = g.st.slot_pokemon(b.source.p as usize, b.source.s);
        let source_card = match source_card {
            Some(c) if pokemon == Some(me) => c,
            _ => return Ok(()),
        };
        if t.p == b.source.p {
            return Ok(());
        }
        if g.st.phase != GamePhase::Attack {
            return Ok(());
        }
        if g.st.cdef(source_card).has_tag(tag::POKEMON_EX_LOWER) {
            let player = t.p;
            let power = PowerRef { card: me, index: 0 };
            if g.run_fx(Effect::Power { p: player, power, card: me, target: None, probe: false }).is_err() {
                return Ok(());
            }
            g.set_prevent(e, true);
        }
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let o = 1 - p as usize;
        let target = SlotRef::new(o, g.st.players[o].active);
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
        let (w, _) = g.run_fx(Effect::ApplyWeakness { b, damage: 120, ignore_weakness: false, ignore_resistance: false })?;
        let damage = match w {
            Effect::ApplyWeakness { damage, .. } => damage,
            _ => 120,
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 0;
        }
        if damage > 0 {
            let a = g.st.players[o].active;
            g.st.players[o].slots[a as usize].damage += damage;
            let target = SlotRef::new(o, a);
            let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
            g.run_fx(Effect::AfterDamage { b, damage })?;
        }
    }
    Ok(())
}
