//! Dangerous Laser (SFA, ACE SPEC): your opponent's Active Pokémon is now Burned and Confused.
//!
//! Two Special Condition events on the opponent's Active Pokémon caused by this Item, so a Pokémon that can't be affected by
//! Special Conditions (or a Stadium effect that blocks Trainer effects) keeps them out (RULES.md events batch 4). It can be
//! played when at least one of the two conditions would change (`WouldChangeConditions`, id290; JP FAQ Dangerous Laser).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DangerousLaser",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::WouldChangeConditions(OPP_ACTIVE, &[SpecialCondition::Burned, SpecialCondition::Confused])],
        steps: &[
            Step::new(Op::Conditions(ConditionsSpec { target: OPP_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Burned, SpecialCondition::Confused]), when: Cond::True })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
