//! Mega Latias ex (MEG 100): Strafe - 40; you may switch this Pokémon with 1 of your Benched Pokémon. Illusory Impulse -
//! 300; discard all Energy from this Pokémon.
//!
//! The choice is asked even with nothing to switch with; yes is a ChangeActive (Switch, your own, APR C-03). Illusory
//! Impulse discards every Energy card on the Pokémon after the damage: a LeavePlay per card (D1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaLatiasex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                // The choice is asked even with nothing to switch with (the attack stays usable, APR D-01).
                when: Cond::True,
                msg: "WANT_TO_SWITCH_POKEMON",
                yes: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Bench, msg: "CHOOSE_NEW_ACTIVE_POKEMON", required: false }))],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
