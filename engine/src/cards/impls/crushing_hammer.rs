//! Crushing Hammer (PBL 105): Item. Flip a coin. If heads, discard an Energy attached to 1 of your opponent's Pokémon.
//!
//! On heads the Energy is a LeavePlay (D1) caused by this Item, to its owner's discard pile; on tails nothing happens.
//! The card needs an opposing Pokémon with an Energy attached (a Trainer that can't have any effect can't be played,
//! id925).
use crate::spec::prelude::*;

const TARGETS: SlotSel = SlotSel::Pokemon(Who::Opp);
const ATTACHED: ZoneRef = ZoneRef(Who::Opp, Zone::AttachedEnergy(SlotExpr::Picked));

pub static SPEC: CardSpec = CardSpec {
    class: "CrushingHammer",
    // Flip a coin; if heads, discard an Energy attached to 1 of your opponent's Pokémon.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::Coin(CoinSpec {
            heads: &[
                Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&TARGETS, SlotPred::HasEnergy) }, purpose: PromptPurpose::DiscardCard })),
                Step::new(Op::Pick(PickSpec {
                    from: ATTACHED,
                    predicate: Pred::Energy,
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    into: 0,
                    ..PickSpec::DEFAULT
                })),
                Step::new(Op::Discard(DiscardSpec { from: ATTACHED, cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            ],
            ..CoinSpec::DEFAULT
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
