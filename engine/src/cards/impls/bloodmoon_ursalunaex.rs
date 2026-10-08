//! Bloodmoon Ursaluna ex (TWM): Seasoned Skill — Blood Moon costs [C] less
//! for each Prize card your opponent has taken. Blood Moon — 240; during
//! your next turn, this Pokémon can't attack.
//!
//! Twinleaf: the reduction runs on CheckAttackCostEffect for this card's
//! attack (after the ability-lock probe, before the [C] check), removing
//! 6 - prizesLeft [C] (none when the opponent has 6 or 0 prizes left).
//! Blood Moon sets `cannotAttackNextTurnPending` on the player's Active.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BloodmoonUrsalunaex",
attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Blood Moon costs [C] less for each Prize card your opponent has taken.
        modifier: Modifier::AttackCost(AttackCostSpec {
            change: CostChange::Reduce(Num::If(
                &Cond::All(&[Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Ge, Num::Lit(1)), Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(5))]),
                &Num::Sub(&Num::Lit(6), &Num::PrizesLeft(Who::Opp)),
                &Num::Lit(0),
            )),
            attack: Some(0),
            subject: SlotPred::Any,
            side: Side::Owner,
            ..AttackCostSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
