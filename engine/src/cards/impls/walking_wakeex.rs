//! Walking Wake ex (TEF): Azure Wave — damage from attacks used by this
//! Pokémon isn't affected by any effects on your opponent's Active Pokémon.
//! Cathartic Roar — 120+; if your opponent's Active Pokémon is affected by a
//! Special Condition, this attack does 120 more damage.
//!
//! Twinleaf: on EVERY AttackEffect (any copy of this card, wherever it is)
//! whose player has a Pokémon named 'Walking Wake ex' Active: a probe
//! PowerEffect for this copy that throws skips the whole handler (including
//! Cathartic Roar's bonus); otherwise `effect.attack.shredAttack = true` is
//! written on the used attack's object (visible in the canonical `cards`
//! entry of the attacking card) and the damage is applied by the shared
//! "not affected by effects" pattern (own ApplyWeaknessEffect, damage zeroed,
//! added straight to the opponent's Active, AfterDamageEffect). Cathartic
//! Roar then adds 120 to the zeroed damage when the opponent's Active has any
//! Special Condition.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "WalkingWakeex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, opp, damage, attack) = match *g.e(e) {
        Effect::Attack { p, opp, damage, attack, .. } => (p as usize, opp as usize, damage, attack),
        _ => return Ok(()),
    };
    let active = g.st.active_pokemon(p);
    if let Some(c) = active {
        if g.st.cdef(c).name == "Walking Wake ex" {
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            // `effect.attack.shredAttack = true` (a per-card attack object write).
            // A copy-attack clone carries its own flag (nothing reads it).
            if !attack.is_clone() && !g.st.cdef(attack.card).attacks[attack.idx()].shred_attack {
                g.st.cards[attack.card as usize].attack_shred |= 1u8 << attack.idx();
            }
            super::mega_lopunnyex::shred(g, e, damage)?;
        }
    }

    if was_attack_used(g, e, 0, me) {
        let oa = g.st.players[opp].active;
        if !g.st.slot(opp, oa).special_conditions.is_empty() {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 120;
            }
        }
    }
    Ok(())
}
