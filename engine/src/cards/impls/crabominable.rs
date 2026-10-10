//! Crabominable (SCR): Food Prep — attacks used by this Pokémon cost [C] less
//! for each Kofu card in your discard pile. Haymaker — 250; during your next
//! turn, this Pokémon can't use Haymaker.
//!
//! Twinleaf: on CheckAttackCostEffect (any attack of that player's Active),
//! when this is the player's Active and a PowerEffect for Food Prep passes,
//! `cost.splice(cost.indexOf(C), kofuCount)` — with no [C] left the index is
//! -1, so the last entry is removed (at most one).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Crabominable",
attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Attacks cost [C] less for each Kofu card in your discard pile.
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::Reduce(Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::Trainer, Pred::Name("Kofu")]))),
            subject: SlotPred::IsThisPokemon,
            side: Side::Owner,
            ..AttackCostSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
