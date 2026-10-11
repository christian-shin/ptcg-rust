//! Mega Dragonite ex (M2a / ASC 152): Sky Transport - once during your turn, switch your Active Pokémon with 1 of your Benched
//! Pokémon. Ryuno Glide - 330; discard 2 Energy from this Pokémon.
//!
//! Ryuno Glide's discard is an effect of the attack after the damage: a LeavePlay (to the discard pile) of 2 Energy attached
//! to it, caused by this attack (D1).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaDragoniteex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: crate::types::ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT }))],
    }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("SKY_CARRY_MARKER"),
        needs: &[],
        steps: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Bench, msg: "CHOOSE_NEW_ACTIVE_POKEMON", required: true }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
