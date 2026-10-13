//! Mega Manectric ex (M1S): Flash Ray — 120; during your opponent's next
//! turn, prevent all damage done to this Pokémon by attacks from Basic
//! Pokémon. Riotous Blasting — 200+; you may discard all Energy from this
//! Pokémon for 130 more damage.
//!
//! Flash Ray arms `Lasting::PreventDamage` for attacks from Basic Pokémon (a `Prevent` over damage, step 6, APR C-16) through the
//! opponent's next turn. Riotous Blasting is a `May` before the damage: yes adds 130 and discards all Energy from this Pokémon; the
//! 130 more is done even when there was no Energy to discard (JP FAQ Mega Manectric ex).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaManectricEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Stage(crate::types::Stage::Basic)) }))] },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                msg: "WANT_TO_DISCARD_ENERGY",
                yes: &[
                    Step::new(more_damage_if(130, Cond::True)),
                    Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::This), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
                ],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
