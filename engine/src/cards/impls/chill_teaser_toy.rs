//! Chill Teaser Toy (SSP 166, Item): you can use this card only if you go second, and only during your first turn. Put an Energy
//! attached to 1 of your opponent's Pokémon into their hand.
//!
//! The condition is printed (`Cond::Printed`): the turn is the game's second (the turn of the player who goes second, who has
//! no other first turn). The Energy is chosen from a Pokémon of the opponent's that has one; it leaves play into its owner's
//! hand (a LeavePlay caused by this Item, APR C-02). With no Energy on the opponent's Pokémon it can't be played (id925).
use crate::spec::prelude::*;

const TARGETS: SlotSel = SlotSel::Pokemon(Who::Opp);
const ATTACHED: ZoneRef = ZoneRef(Who::Opp, Zone::AttachedEnergy(SlotExpr::Picked));

pub static SPEC: CardSpec = CardSpec {
    class: "ChillTeaserToy",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Printed(&Cond::Cmp(Num::Turn, CmpOp::Eq, Num::Lit(2)))],
        steps: &[
            Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&TARGETS, SlotPred::HasEnergy) }, purpose: PromptPurpose::DiscardCard })),
            Step::new(Op::Pick(PickSpec { from: ATTACHED, predicate: Pred::Energy, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ATTACHED, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
