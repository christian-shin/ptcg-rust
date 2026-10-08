//! Mega Venusaur ex (M1L 3): Solar Trans — as many times as you like during
//! your turn, move a Basic [G] Energy from one of your Pokémon to another.
//! Jungle Dump — 240; heal 30 damage from this Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaVenusaurEx",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(heal_active(30, HealVia::Effect))] }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[
            Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::HasCard(Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]))),
            Cond::Cmp(Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::Any), CmpOp::Ge, Num::Lit(2)),
        ],
        steps: &[Step::new(Op::MoveEnergy(MoveEnergySpec { chooser: Who::Me, owner: Who::Me, mode: MoveEnergyMode::BasicNamed { name: "Grass Energy" } }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
