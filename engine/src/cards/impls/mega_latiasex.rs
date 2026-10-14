//! Mega Latias ex (MEG 100): Strafe - 40; you may switch this Pokémon with 1 of your Benched Pokémon. Illusory Impulse - 300;
//! discard all Energy from this Pokémon.
//!
//! The choice is a `May` asked only with a Benched Pokémon (`when`): with none the yes can't change the game, so it isn't asked
//! (APR B-01, id925, user decision 2a; scenario mega-latias-ex-meg-100-strafe-without-a-bench-asks-nothing). Yes is a ChangeActive (Switch, your own, APR C-03).
//! Illusory Impulse discards every Energy card on the Pokémon after the damage: a LeavePlay per card (D1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaLatiasex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                // Asked only with a Benched Pokémon to switch with: with none the yes can't change the game (APR B-01, id925); the
                // attack stays usable (APR D-01).
                when: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any),
                purpose: PromptPurpose::MaySwitch,
                yes: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Bench, purpose: PromptPurpose::NewActive, required: false }))],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
