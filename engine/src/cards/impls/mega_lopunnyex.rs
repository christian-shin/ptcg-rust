//! Mega Lopunny ex (PFL / M2): Gale Thrust — 60+, 170 more if this Pokémon
//! moved from your Bench to the Active Spot this turn. Spiky Hopper — 160,
//! not affected by effects on your opponent's Active Pokémon.
//!
//! Twinleaf: Gale Thrust checks `player.movedToActiveThisTurn` for this
//! card's id. Spiky Hopper sets `ignoreDefenderEffects` (phase 4b R7B: it used
//! to add the damage straight to the Active, skipping the attacker's effects too;
//! now the normal damage path skips only the effects on the Defending Pokémon).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaLopunnyex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            if g.st.players[p as usize].moved_to_active_this_turn.contains(&me) {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 170;
                }
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        shred(g, e, 160)?;
    }
    Ok(())
}

/// `THIS_ATTACKS_DAMAGE_ISNT_AFFECTED_BY_EFFECTS` without the Weakness/Resistance flag.
pub fn shred(g: &mut Game, e: EffId, base: i32) -> R {
    shred_ex(g, e, base, false)
}

/// `THIS_ATTACKS_DAMAGE_ISNT_AFFECTED_BY_EFFECTS(..., ignoreWeaknessAndResistance)`:
/// sets `AttackEffect.ignoreDefenderEffects` (and, with the flag, ignores Weakness and
/// Resistance: "isn't affected by Weakness or Resistance, or by any effects"). The damage then goes
/// through the normal DealDamage / PutDamage path, which applies Weakness, Resistance and the effects on
/// the attacker but skips every effect on the damaged Pokémon (rulings 1439, 1716, 1816, 531, 532).
pub fn shred_ex(g: &mut Game, e: EffId, _base: i32, ignore_wr: bool) -> R {
    if let Effect::Attack { ignore_defender_effects, ignore_weakness, ignore_resistance, .. } = g.e_mut(e) {
        *ignore_defender_effects = true;
        if ignore_wr {
            *ignore_weakness = true;
            *ignore_resistance = true;
        }
    }
    Ok(())
}
