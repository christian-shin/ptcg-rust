//! Tool Scrapper (DRX): choose up to 2 Pokémon Tools attached to Pokémon in
//! play (yours or your opponent's) and discard them. Phase 4b (rulings 1778/1853): the prompt can't
//! be cancelled (a cancel was choosing 0).
//!
//! Fixed (phase 4b, F1): the choice is over the Tools (one DiscardEnergyPrompt with a Pokémon Tool
//! filter, min 1, max min(2, Tools in play)), not over Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ToolScrapper",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Cmp(Num::ToolsInPlay, CmpOp::Gt, Num::Lit(0))],
        // Choose up to 2 Pokémon Tools attached to Pokémon in play (yours or your opponent's) and discard them.
        steps: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Tools { min: Num::Lit(1), max: Num::Min(&Num::Lit(2), &Num::ToolsInPlay) }, to: EnergyDest::Discard, ..DiscardEnergySpec::DEFAULT }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
