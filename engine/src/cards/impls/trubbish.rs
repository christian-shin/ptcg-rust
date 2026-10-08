//! Trubbish (M4 / CRI 56): Acid Spray — 10; flip a coin, if heads discard an
//! Energy attached to your opponent's Active Pokémon.
//!
//! Twinleaf: DISCARD_AN_ENERGY_FROM_OPPONENTS_ACTIVE_POKEMON in the coin
//! callback (no prompt when the Active has no Energy card).
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
