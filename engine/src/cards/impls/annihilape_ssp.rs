//! Annihilape (SSP): Tantrum — 130; this Pokémon is now Confused. Destined
//! Fight — both Active Pokémon are Knocked Out.
//!
//! Twinleaf (surging-sparks file): Tantrum is ADD_CONFUSION_TO_PLAYER_ACTIVE
//! (an AddSpecialConditionsPowerEffect on `player.active`); Destined Fight
//! reduces KnockOutPlayerEffect on `player.active` (the opponent takes the
//! Prizes) and then KnockOutOpponentEffect on `opponent.active`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Annihilape@SSP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Confused]), cause: Cause::Ability, gate: Gate::None, when: Cond::True })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::KnockOut(KnockOutSpec { target: MY_ACTIVE, mode: KnockOutMode::Player, when: Cond::True })),
                Step::after_damage(Op::KnockOut(KnockOutSpec { target: OPP_ACTIVE, mode: KnockOutMode::Opponent, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
