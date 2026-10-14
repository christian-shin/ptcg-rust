//! Strange Timepiece (MEG 128, Item): devolve 1 of your evolved [P] Pokémon by putting any number of Evolution cards on it into your
//! hand. (That Pokémon can't evolve this turn.)
//!
//! The Item needs an evolved [P] Pokémon. One is chosen, then the Evolution cards to put into the hand (never the Basic Pokémon at the
//! bottom): a devolve into the hand is a LeavePlay of each Evolution card (Milotic's Mentally Calm stops it). The parenthetical is
//! reminder text for the normal turn rules, so a permission such as Boosted Evolution lifts it (id2327, RULES.md).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StrangeTimepiece",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        // One of your evolved [P] Pokémon.
        needs: &[Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Evolved, SlotPred::TypeIs(ct::PSYCHIC)]))],
        steps: &[
            Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Evolved, SlotPred::TypeIs(ct::PSYCHIC)])) }, purpose: PromptPurpose::Devolve })),
            Step::new(Op::Devolve(DevolveSpec { slot: SlotExpr::Picked, destination: ZoneRef(Who::Me, Zone::Hand), chooser: Some(Who::Me) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
