//! N's Zekrom (M2a / ASC): Shred - 70, not affected by effects on the
//! Defending Pokémon; Rampaging Thunder - 250, can't attack next turn.
//!
//! Twinleaf quirk kept: Shred builds its own ApplyWeaknessEffect, whose
//! `ignoreResistance` defaults to false, so Weakness and Resistance both
//! apply; the damage is added directly, followed by an AfterDamageEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NsZekrom", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        if let Effect::Attack { ignore_resistance, .. } = g.e_mut(e) {
            *ignore_resistance = true;
        }
        let o = opp as usize;
        let target = SlotRef::new(o, g.st.players[o].active);
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
        let (w, _) = g.run_fx(Effect::ApplyWeakness { b, damage: 70, ignore_weakness: false, ignore_resistance: false })?;
        let damage = match w {
            Effect::ApplyWeakness { damage, .. } => damage,
            _ => 70,
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 0;
        }
        if damage > 0 {
            let a = g.st.players[o].active;
            g.st.players[o].slots[a as usize].damage += damage;
            let target = SlotRef::new(o, a);
            g.run_fx(Effect::AfterDamage { b: AtkBase { target, ..b }, damage })?;
        }
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let pl = &mut g.st.players[p as usize];
            let a = pl.active;
            pl.slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    Ok(())
}
