//! Mega Latias ex (M1S / MEG 100): Strafe — 40; you may switch this Pokémon
//! with 1 of your Benched Pokémon. Illusory Impulse — 300; discard all Energy
//! from this Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaLatiasex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                // Asked even with nothing to switch with (Twinleaf).
                when: Cond::True,
                msg: "WANT_TO_SWITCH_POKEMON",
                yes: &[Step::new(Op::Switch(SwitchSpec { side: Who::Me, chooser: Who::Me, kind: SwitchKind::Plain, msg: "CHOOSE_NEW_ACTIVE_POKEMON", required: false }))],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
