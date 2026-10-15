//! Metagross (CRI 61): Bounce Back - 60; switch out your opponent's Active Pokémon to the Bench (your opponent chooses the
//! new Active Pokémon). Metallic Hammer - 150+; you may discard 3 [M] Energy from this Pokémon and have this attack do 150
//! more damage.
//!
//! Bounce Back is a ChangeActive (SwitchOut, APR C-05) after the damage; the opponent chooses the new Active Pokémon.
//! Metallic Hammer: the choice is offered even without 3 [M] Energy (id2352); yes adds 150 before the damage and the
//! Energy (as many as there are, up to 3) is discarded after it: a LeavePlay per card (D1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MetagrossCRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Switch(SwitchSpec { change: ActiveChange::SwitchOut, among: SwitchAmong::Bench, required: false }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                // Offered even without 3 [M] Energy (id2352): as many as there are are discarded.
                yes: &[
                    Step::new(more_damage_if(150, Cond::True)),
                    Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 3, ty: crate::types::ct::METAL }, ..DiscardEnergySpec::DEFAULT })),
                ],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
