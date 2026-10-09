//! Iron Leaves ex (TEF): Rapid Vernier - when you play this Pokémon from
//! your hand onto your Bench, you may switch it with your Active Pokémon;
//! if you do, you may move any number of Energy from your Benched Pokémon
//! to it. Prism Edge - 180; this Pokémon can't attack during your next turn.
//!
//! Rule: Rapid Vernier triggers after this Pokémon is on the Bench (EnterPlay
//! by the rule from the hand). The switch is a ChangeActive (Switch, APR C-03)
//! by the Ability, done to the Active Pokémon; "if you do" reads its outcome
//! (`Cond::Done`), and each Energy moved is a MoveEnergy from a Benched Pokémon
//! to this one.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IronLeavesex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn })),
        ] },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EnterPlay), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand), EventPred::Mode(EnterMode::Rule), EventPred::Slot(SlotPred::IsBench)])), steps: &[Step::new(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::If(IfSpec { cond: Cond::Not(&Cond::AbilityBlocked), yes: &[Step::new(Op::SwitchWithActive(SwitchWithActiveSpec { target: SlotExpr::This })), Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::MoveEnergy(MoveEnergySpec { chooser: Who::Me, owner: Who::Me, mode: MoveEnergyMode::BenchToActive { max: None, required: false, ability: true } }))], no: &[] }))], no: &[] }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
