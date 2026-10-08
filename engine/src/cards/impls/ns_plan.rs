//! N's Plot (SV11B, Twinleaf class NsPlan): move up to 2 Energy from your
//! Benched Pokémon to your Active Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsPlan",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::HasEnergy)],
        steps: &[Step::new(Op::MoveEnergy(MoveEnergySpec { chooser: Who::Me, owner: Who::Me, mode: MoveEnergyMode::BenchToActive { max: 2 } }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
