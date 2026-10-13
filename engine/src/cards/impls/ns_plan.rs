//! N's Plan (BLK 83): move up to 2 Energy from your Benched Pokémon to your Active Pokémon.
//!
//! `Op::MoveEnergy` (BenchToActive, up to 2): the player chooses the Energy to move, each a MoveEnergy of the Supporter. It
//! can't be played when no Benched Pokémon has Energy (a Trainer that can't have any effect, id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsPlan",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::HasEnergy)],
        steps: &[Step::new(Op::MoveEnergy(MoveEnergySpec { chooser: Who::Me, owner: Who::Me, mode: MoveEnergyMode::BenchToActive { max: Some(2), required: true, ability: false } }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
