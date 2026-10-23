//! Dewgong (POR 19): Wash Out - as often as you like during your turn, you may use this Ability. Move a [W] Energy from 1 of your
//! Benched Pokémon to your Active Pokémon. Wave Splash - 60.
//!
//! An Ability usable any number of times: one Energy card that provides [W] (a Special Energy that provides [W] too: id2312) moves
//! from a Benched Pokémon to the Active Pokémon (a MoveEnergy event, this Ability its cause; Cobalion ex CRI 64's Metal Road, with
//! one Energy). It needs a Benched Pokémon holding such an Energy card.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DewgongPOR",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::HasCard(Pred::All(&[Pred::Energy, Pred::Provides(ct::WATER)])))],
        steps: &[Step::new(Op::MoveEnergy(MoveEnergySpec {
            chooser: Who::Me,
            owner: Who::Me,
            mode: MoveEnergyMode::BenchToActive { max: Some(1), required: true, ability: true, only: Some(ct::WATER) },
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
