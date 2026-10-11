//! Zapdos (TWM 65): Thunder Wave: flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed. Thunderbolt: 190;
//! discard all Energy from this Pokémon.
//!
//! Thunder Wave is a coin flip, on heads a GainCondition (Paralyzed) on the opponent's Active (this attack its cause).
//! Thunderbolt discards after the damage (id2423's order: damage first, then the discard; LeavePlay events, this attack their
//! cause), so the Energy counts for the damage (Voltaic Lightning Energy's +20).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zapdos",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
