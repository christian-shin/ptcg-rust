//! Croconaw (ASC 42): Crunch - 50; flip a coin. If heads, discard an Energy from your opponent's Active Pokémon.
//!
//! On heads one Energy chosen from the opponent's Active Pokémon is discarded after the damage (Goldeen).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CroconawASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Opp)), selection: EnergySelection::Chosen { pred: Pred::Energy }, ..DiscardEnergySpec::DEFAULT }))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
