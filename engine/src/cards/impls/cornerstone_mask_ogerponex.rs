//! Cornerstone Mask Ogerpon ex (TWM 112): Cornerstone Stance — prevent all
//! damage done to this Pokémon by attacks from your opponent's Pokémon that
//! have an Ability. Demolish — 140, not affected by Weakness, Resistance or
//! effects on the opponent's Active. Tera.
//!
//! Twinleaf: Demolish sets `ignoreDefenderEffects` and ignores Weakness and
//! Resistance on the AttackEffect (phase 4b R7B: it used to add 140 straight
//! to the Active, skipping the attacker's effects). Cornerstone
//! Stance checks the source's printed `powers` (any kind) and an ability
//! probe for this card's owner. The Tera bench protection sits after it and
//! is skipped by its early returns (this card not the top card, no source
//! Pokémon, own damage, outside the attack phase, or the ability blocked).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "CornerstoneMaskOgerponex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        // Demolish: not affected by Weakness, Resistance or effects on the Defending Pokémon.
        if let Effect::Attack { ignore_defender_effects, ignore_weakness, ignore_resistance, .. } = g.e_mut(e) {
            *ignore_defender_effects = true;
            *ignore_weakness = true;
            *ignore_resistance = true;
        }
    }

    if let Effect::PutDamage { b, .. } = *g.e(e) {
        if ignores_defender_effects(g, &b) {
            return Ok(());
        }
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).cards.contains(me) {
            let source_card = g.st.slot_pokemon(b.source.p as usize, b.source.s);
            if g.st.slot_pokemon(t.p as usize, t.s) != Some(me) {
                return Ok(());
            }
            let source_card = match source_card {
                Some(c) => c,
                None => return Ok(()),
            };
            if t.p == b.source.p {
                return Ok(());
            }
            if g.st.phase != GamePhase::Attack {
                return Ok(());
            }
            if !g.st.cdef(source_card).powers.is_empty() {
                if is_ability_blocked(g, t.p as usize, me, None) {
                    return Ok(());
                }
                g.set_prevent(e, true);
            }
        }
    }

    if let Effect::PutDamage { .. } = *g.e(e) {
        tera_rule(g, e, me);
    }
    Ok(())
}
