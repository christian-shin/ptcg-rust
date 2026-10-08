//! Metagross (M4 / CRI): Bounce Back — 60, after attacking switch out your
//! opponent's Active Pokémon (your opponent chooses the new Active Pokémon;
//! fixed in phase 4b: Twinleaf let the attacker choose and switched without
//! dispatching the switch effects). Metallic Hammer — 150+, you may discard 3
//! [M] Energy from this Pokémon for 150 more damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Metagross@CRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Opp, kind: SwitchKind::SwitchOut, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                msg: "WANT_TO_DISCARD_ENERGY",
                // Offered even without 3 [M] Energy (ruling 1822): as many as there are are discarded.
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
