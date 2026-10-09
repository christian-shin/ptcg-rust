//! Jumbo Ice Cream (PFL / M2): heal 80 damage from your Active Pokémon that
//! has 3 or more Energy attached.
//!
//! Twinleaf: phase 4b (R6): throws CANNOT_PLAY_THIS_CARD when the Active has
//! no damage or fewer than 3 Energy attached (it used to just discard the card
//! with no effect); the Energy count is the number of provided-energy
//! entries; the card moves itself from the supporter pile to the discard.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "JumboIce",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Slot(SlotExpr::Active(Who::Me), SlotPred::Damaged), Cond::Cmp(Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Me)), EnergyUnit::ProvidedCards), CmpOp::Ge, Num::Lit(3))],
        steps: &[
            Step::new(heal_active(80)),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
