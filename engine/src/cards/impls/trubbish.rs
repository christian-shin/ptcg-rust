//! Trubbish (CRI 56): Acid Spray - 10; flip a coin. If heads, discard an Energy from your opponent's Active Pokémon.
//!
//! On heads the Energy is a LeavePlay (D1) caused by this attack, to its owner's discard pile (the attacker chooses
//! which); on tails nothing happens; an Active Pokémon without Energy loses none.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Trubbish",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: false }, ..DiscardEnergySpec::DEFAULT }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
