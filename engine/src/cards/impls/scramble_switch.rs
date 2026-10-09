//! Scramble Switch (PLS, ACE SPEC): switch your Active Pokémon with 1 of your
//! Benched Pokémon. Then, you may move as many Energy attached to the old
//! Active Pokémon to the new Active Pokémon as you like.
//!
//! Not playable without a Benched Pokémon; the choice can't be cancelled (phase 4b, R3). The Energy chosen moves
//! to the chosen Benched Pokémon first (MoveEnergy), then the switch (a ChangeActive, APR C-03).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ScrambleSwitch",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Cmp(Num::BenchCount(Who::Me), CmpOp::Gt, Num::Lit(0))],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Me), msg: "CHOOSE_POKEMON_TO_SWITCH" })),
            // You may move as many Energy as you like from the old Active Pokémon to the new one.
            Step::new(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Cards { min: Num::Lit(0), max: Num::ZoneSize(ZoneRef(Who::Me, Zone::Attached(MY_ACTIVE))), kind: EnergyKind::Any, cancel: false, energies_only: false }, to: EnergyDest::Slot(SlotExpr::Picked), ..DiscardEnergySpec::DEFAULT })),
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Picked, msg: "", required: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
