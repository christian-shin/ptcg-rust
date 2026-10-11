//! Turtonator (SSP): Fully Singe — discard an Energy from your opponent's Active Pokémon ex. Steaming Stomp — 100.
//!
//! When the opponent's Active Pokémon is an ex with an Energy attached, one Energy is chosen (no cancel) and leaves the
//! Pokémon in a LeavePlay caused by the attack, after the damage.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Turtonator",
    attacks: &[AttackSpec {
        index: 0,
        // Fully Singe: discard an Energy from your opponent's Active Pokémon ex.
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::Tag(tag::POKEMON_EX_LOWER))),
            yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: false }, ..DiscardEnergySpec::DEFAULT }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
