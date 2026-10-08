//! Mega Dragonite ex (M2a / ASC 152): Sky Transport — once during your turn,
//! switch your Active Pokémon with 1 of your Benched Pokémon. Ryuno Glide —
//! 330; DISCARD_X_ENERGY_FROM_THIS_POKEMON(2).
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
        steps: &[Step::new(Op::Switch(SwitchSpec { side: Who::Me, chooser: Who::Me, kind: SwitchKind::Plain, msg: "CHOOSE_NEW_ACTIVE_POKEMON", required: true }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
