//! Iron Leaves ex (TEF): Rapid Vernier - when you play this Pokémon from
//! your hand onto your Bench, you may switch it with your Active Pokémon;
//! if you do, you may move any number of Energy from your Benched Pokémon
//! to it. Prism Edge - 180; this Pokémon can't attack during your next turn.
//!
//! Twinleaf quirks kept: the confirm prompt is created before the card is
//! benched (and before the ability-lock probe, which runs in the callback);
//! the energy move loops over every transfer once per transfer, always
//! using the outer transfer's source (moves of cards not in that source are
//! no-ops, but each still reduces a MoveCardsEffect).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IronLeavesex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn })),
        ] },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EnterPlay), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand), EventPred::Mode(EnterMode::Rule), EventPred::Slot(SlotPred::IsBench)])), steps: &[Step::new(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::If(IfSpec { cond: Cond::Not(&Cond::AbilityBlocked), yes: &[Step::new(Op::SwitchWithActive(SwitchWithActiveSpec { target: SlotExpr::This })), Step::new(Op::MoveEnergy(MoveEnergySpec { chooser: Who::Me, owner: Who::Me, mode: MoveEnergyMode::BenchToActive { max: None, required: false, ability: true } }))], no: &[] }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
