//! Raichu (ASC 56): Quick Blow - 20+; flip a coin. If heads, this attack does 50 more damage. Strong Volt - 150; discard a
//! [L] Energy from this Pokémon.
//!
//! A coin before the damage; heads adds 50 (Boltund PFL 31). Strong Volt discards one [L] Energy this Pokémon provides
//! after the damage (a LeavePlay event caused by the attack; Metagross TEF 115's Luster Blast with a type).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RaichuASC",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(50, Cond::True))], ..CoinSpec::DEFAULT }))] },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 1, ty: ct::LIGHTNING, up_to: false }, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
