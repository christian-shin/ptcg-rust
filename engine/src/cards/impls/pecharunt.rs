//! Pecharunt (SVP): Toxic Subjugation - while this Pokémon is Active, your
//! opponent's Poisoned Pokémon takes 5 more damage counters during Checkup.
//! Poison Chain - 10; the opponent's Active is now Poisoned and can't
//! retreat during their next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Pecharunt",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(inflict(&[SpecialCondition::Poisoned], Cause::Attack)), Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventRetreat }))],
    }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::CheckupDamage(CheckupDamageSpec { amount: 50, victim: SlotPred::Condition(SpecialCondition::Poisoned), opponent_only: true, holder: SlotPred::IsActive }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
