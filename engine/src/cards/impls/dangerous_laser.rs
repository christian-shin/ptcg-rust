//! Dangerous Laser (SFA, ACE SPEC): your opponent's Active Pokémon is now
//! Burned and Confused.
//!
//! Twinleaf: unless a TrainerTargetEffect on the Active is blocked, the
//! conditions are added directly (`addSpecialCondition`, no effect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DangerousLaser",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::WouldChangeConditions(OPP_ACTIVE, &[SpecialCondition::Burned, SpecialCondition::Confused])],
        steps: &[
            Step::new(Op::Conditions(ConditionsSpec { target: OPP_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Burned, SpecialCondition::Confused]), gate: Gate::TrainerTarget, when: Cond::True })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
