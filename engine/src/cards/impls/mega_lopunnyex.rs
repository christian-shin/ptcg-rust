//! Mega Lopunny ex (PFL / M2): Gale Thrust — 60+, 170 more if this Pokémon
//! moved from your Bench to the Active Spot this turn. Spiky Hopper — 160,
//! not affected by effects on your opponent's Active Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaLopunnyex",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(170, Cond::ThisMovedToActive))] },
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// Kept for Walking Wake ex, Dudunsparce ex and Jirachi ex (still hand-written); delete with their conversions.
use crate::cards::prelude::*;

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
