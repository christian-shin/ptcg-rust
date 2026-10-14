//! Enhanced Hammer (TWM): discard a Special Energy attached to 1 of your opponent's Pokémon.
//!
//! The Discard of an attached card is a LeavePlay of the Item's effect: the Energy goes from the Pokémon to its owner's
//! discard pile. It needs a Pokémon of the opponent with a Special Energy attached (else it has no effect: id925).
use crate::spec::prelude::*;

const ATTACHED: ZoneRef = ZoneRef(Who::Opp, Zone::AttachedEnergy(SlotExpr::Picked));

pub static SPEC: CardSpec = CardSpec {
    class: "EnhancedHammer",
    // Discard a Special Energy attached to 1 of your opponent's Pokémon.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec {
                chooser: Who::Me,
                among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::HasSpecialEnergy),
                purpose: PromptPurpose::DiscardAttachedFrom,
            })),
            Step::new(Op::Pick(PickSpec {
                from: ATTACHED,
                predicate: Pred::SpecialEnergy,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                purpose: Some(PromptPurpose::DiscardCard),
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Discard(DiscardSpec { from: ATTACHED, cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
