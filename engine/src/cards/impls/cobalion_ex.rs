//! Cobalion ex (CRI 64): Metal Road - once during your turn, when this Pokémon moves from your Bench to the Active Spot, you may use
//! this Ability. Move any amount of [M] Energy from your other Pokémon to this Pokémon. Power Tackle - 200; during your next turn,
//! this Pokémon can't use attacks.
//!
//! Metal Road is a trigger over the ChangeActive event that brings this Pokémon to the Active Spot during its owner's turn (a
//! retreat, a switch, a promotion: APR E-25), unless the Ability's usage record is there: a question; no -> record; yes -> record, then
//! any number of Energy cards that provide [M] move from the Benched Pokémon to this one (a MoveEnergy event per card, caused by the
//! Ability; Yanmega ex's trigger, Iron Leaves ex's move). A blocked Ability asks nothing and sets nothing. Power Tackle arms the
//! lasting lock on this Pokémon over every attack through the end of your next turn (Cubchoo BLK 25 locks the Defending Pokémon).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CobalionEx",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::ChangeActive), EventPred::To(SlotPred::IsThisPokemon), EventPred::Turn(TurnOf::Me)])),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::All(&[Cond::Not(&Cond::Used(UsedSpec::ability(0, UsedBy::Copy))), Cond::Not(&Cond::AbilityBlocked)]),
            yes: &[Step::new(Op::If(IfSpec {
                cond: Cond::Not(&Cond::AbilityBlocked),
                yes: &[
                    Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Copy))),
                    Step::new(Op::MoveEnergy(MoveEnergySpec {
                        chooser: Who::Me,
                        owner: Who::Me,
                        mode: MoveEnergyMode::BenchToActive { max: None, required: false, ability: true, only: Some(ct::METAL) },
                    })),
                ],
                no: &[],
            }))],
            no: &[Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Copy)))],
        }))],
    }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
