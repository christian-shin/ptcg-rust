//! Larvitar (JTG): Crunch — 20; flip a coin. If heads, discard an Energy from your opponent's Active Pokémon.
//!
//! The coin and the Discard run after the damage; the Energy is chosen (no cancel) and leaves the Defending Pokémon in a
//! LeavePlay caused by the attack.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Larvitar",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Coin(CoinSpec { before: Cond::True, heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Opp)), selection: EnergySelection::Chosen { pred: Pred::Energy }, ..DiscardEnergySpec::DEFAULT }))], ..CoinSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
