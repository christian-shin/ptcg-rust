//! Koraidon (TEF): Primordial Beatdown — 30× for each of your Ancient Pokémon
//! in play. Shred — 130; this attack's damage isn't affected by any effects
//! on your opponent's Active Pokémon.
//!
//! Twinleaf (temporal-forces file): Primordial Beatdown sets
//! `damage = 30 × Ancient Pokémon` (Active + Bench). Shred reduces its own
//! ApplyWeaknessEffect (130, ignore flags unset), zeroes the attack damage,
//! adds the result straight to the opponent's Active and reduces an
//! AfterDamageEffect (no PutDamageEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Koraidon@TEF", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        let mut n = 0;
        for s in std::iter::once(pl.active).chain(pl.bench.iter().copied()) {
            if let Some(c) = g.st.slot_pokemon(p, s) {
                if g.st.cdef(c).has_tag(tag::ANCIENT) {
                    n += 1;
                }
            }
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 30 * n;
        }
    }

    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let o = opp as usize;
        let target = SlotRef::new(o, g.st.players[o].active);
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
        let (w, _) = g.run_fx(Effect::ApplyWeakness { b, damage: 130, ignore_weakness: false, ignore_resistance: false })?;
        let damage = match w {
            Effect::ApplyWeakness { damage, .. } => damage,
            _ => 130,
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
