//! Ignition Energy (PFL / WHT, Special): provides [C]; on an Evolution Pokémon it provides [C][C][C] instead. If this card is
//! attached to 1 of your Pokémon, discard it at the end of your turn.
//!
//! At the end of the owner's turn (a trigger on the EndTurn event: APR E-21, "End of the turn ... is not part of Pokémon
//! Checkup, and takes place just before it", whose example is this card) a card still attached is discarded from its Pokémon (a
//! LeavePlay caused by the card's own rule); the text keeps no record of the attaching. The Pokémon's top card gives the stage: not Basic and not Restored provides
//! [C][C][C] (JP FAQ Ignition Energy).
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "IgnitionEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry { when: SlotPred::Basic, provides: &[ct::COLORLESS] }, ProvidedEntry { when: SlotPred::All(&[SlotPred::Top(Pred::Pokemon), SlotPred::Not(&SlotPred::Basic), SlotPred::Not(&SlotPred::StageIs(Stage::Restored))]), provides: &[ct::COLORLESS, ct::COLORLESS, ct::COLORLESS] }] }) },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Energy, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EndTurn), EventPred::Owner(Who::Me)])), steps: &[Step::new(Op::If(IfSpec { cond: Cond::AnySlot(SlotSel::One(SlotExpr::This), SlotPred::Any), yes: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::This, ..DiscardSpec::DEFAULT }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
