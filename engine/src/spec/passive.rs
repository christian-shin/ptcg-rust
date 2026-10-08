//! Passive modifiers (vocabulary v1 "Passives"): rules that apply while their
//! card is in place, by changing the effects flowing through the engine
//! (`reduce_effect`). A play or use blocker is a passive too, and doubles as
//! the blocker registry declared legality consults.
//!
//! Each modifier declares the effect kinds it reacts to (`kinds`, used for
//! the card's mask) and changes the effect in `apply`.

use crate::list::*;
use crate::effects::{mask, EffId, Effect, KindMask};
use crate::game::{Game, R};
use crate::prefabs::*;

/// A modifier that applies while its card is in place.
pub struct Passive {
    pub origin: RuleSource,
    pub modifier: Modifier,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleSource {
    Ability,
    /// A Pokémon Tool: applies to the Pokémon it is attached to, unless Tools
    /// have no effect.
    Tool,
    Energy,
    Stadium,
    TrainerEffect,
    CardRule,
}

pub enum Modifier {
    DamageDealt(DamageDealtSpec),
    DamageTaken(DamageTakenSpec),
    PreventDamage(PreventDamageSpec),
    PreventAttackEffects(PreventAttackEffectsSpec),
    Prevent(PreventSpec),
    BlockUse(BlockUseSpec),
    AbilityLock(AbilityLockSpec),
    AttackCost(AttackCostSpec),
    RetreatCost(RetreatCostSpec),
    /// +HP to the Pokémon.
    HpBonus(i32),
    SurviveOnTen(SurviveOnTenSpec),
    ProvidesEnergy(ProvidesEnergySpec),
    PrizeAdjust(PrizeAdjustSpec),
    CheckupDamage(CheckupDamageSpec),
    GrantAttacks(GrantAttacksSpec),
    AttackFlags(AttackFlagsSpec),
    StatOverride(StatOverrideSpec),
    EvolveFrom(EvolveFromSpec),
    AllowEvolve(AllowEvolveSpec),
    ConditionImmunity(ConditionImmunitySpec),
    AttachGuard(AttachGuardSpec),
    BenchSize(BenchSizeSpec),
}

pub struct DamageDealtSpec {}
pub struct DamageTakenSpec {}
pub struct PreventDamageSpec {}
pub struct PreventAttackEffectsSpec {}
pub struct PreventSpec {}
pub struct BlockUseSpec {}
pub struct AbilityLockSpec {}
pub struct AttackCostSpec {}
pub struct RetreatCostSpec {}
pub struct SurviveOnTenSpec {}
pub struct ProvidesEnergySpec {}
pub struct PrizeAdjustSpec {}
pub struct CheckupDamageSpec {}
pub struct GrantAttacksSpec {}
pub struct AttackFlagsSpec {}
pub struct StatOverrideSpec {}
pub struct EvolveFromSpec {}
pub struct AllowEvolveSpec {}
pub struct ConditionImmunitySpec {}
pub struct AttachGuardSpec {}
pub struct BenchSizeSpec {}

/// The effect kinds a modifier reacts to.
pub const fn modifier_kinds(m: &Modifier) -> KindMask {
    use crate::effects::k;
    match m {
        Modifier::HpBonus(_) => mask(&[k::CHECK_HP]),
        _ => KindMask::EMPTY,
    }
}

pub(crate) fn apply(g: &mut Game, me: CardId, e: EffId, ps: &Passive) -> R {
    match ps.modifier {
        Modifier::HpBonus(n) => {
            let (p, target, card) = match *g.e(e) {
                Effect::CheckHp { p, target, card } => (p as usize, target, card),
                _ => return Ok(()),
            };
            if !in_place(g, me, p, target, ps.origin) {
                return Ok(());
            }
            // HP is only raised for a Pokémon actually being checked.
            if card.is_some() {
                g.st.players[target.p as usize].slots[target.s as usize].hp_bonus += n;
            }
            Ok(())
        }
        _ => unimplemented!("spec passive not implemented yet (passive.rs)"),
    }
}

/// Does the passive of `me` apply to the Pokémon in `target`?
fn in_place(g: &mut Game, me: CardId, p: usize, target: crate::effects::SlotRef, origin: RuleSource) -> bool {
    match origin {
        RuleSource::Tool => g.st.slot(target.p as usize, target.s).tools.contains(me) && !is_tool_blocked(g, p, me),
        _ => unimplemented!("spec passive origin not implemented yet (passive.rs)"),
    }
}
