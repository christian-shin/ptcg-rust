//! Gastly (TEF 102): Mysterious Beam — flip a coin. If heads, discard an Energy from your opponent's Active Pokémon. Suffocating
//! Gas — 30.
//!
//! The flip comes after the damage (this attack does none); on heads you choose 1 Energy card of the Defending Pokémon and it is
//! discarded (a LeavePlay caused by the attack, which Mist Energy and the like refuse).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GastlyTEF",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec {
                target: SlotTarget::Slot(OPP_ACTIVE),
                selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: true },
                ..DiscardEnergySpec::DEFAULT
            }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
