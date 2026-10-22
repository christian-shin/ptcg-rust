//! Tool Scrapper (CRI / DRX): choose up to 2 Pokémon Tool cards attached to Pokémon in play (yours or your opponent's) and
//! discard them.
//!
//! Each Tool leaves its Pokémon in a LeavePlay of the Item's effect (to its owner's discard pile). The Item needs a Tool in
//! play; as a Trainer's effect "up to 2" takes at least 1 (id2301, id2399), no cancel.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ToolScrapper",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Cmp(Num::ToolsInPlay(None), CmpOp::Gt, Num::Lit(0))],
        // Choose up to 2 Pokémon Tools attached to Pokémon in play (yours or your opponent's) and discard them.
        steps: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Tools { min: Num::Lit(1), max: Num::Min(&Num::Lit(2), &Num::ToolsInPlay(None)) }, to: EnergyDest::Discard, ..DiscardEnergySpec::DEFAULT }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
