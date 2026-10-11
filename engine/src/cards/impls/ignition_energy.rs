//! Ignition Energy (PFL / WHT, Special): provides [C]; on an Evolution Pokémon it provides [C][C][C] instead. If this card is
//! attached to 1 of your Pokémon, discard it at the end of your turn.
//!
//! Attaching it (a trigger on the Attach event of this card) sets the owner's marker; at the end of the turn (the Checkup
//! trigger, `Event::OnCheckup`: B7-OLD -> B8) a card still attached is discarded from its Pokémon (a LeavePlay caused by
//! the card's own rule) and the marker cleared. The Pokémon's top card gives the stage: not Basic and not Restored provides
//! [C][C][C] (JP FAQ Ignition Energy).
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "IgnitionEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry { when: SlotPred::Basic, provides: &[ct::COLORLESS] }, ProvidedEntry { when: SlotPred::All(&[SlotPred::Top(Pred::Pokemon), SlotPred::Not(&SlotPred::Basic), SlotPred::Not(&SlotPred::StageIs(Stage::Restored))]), provides: &[ct::COLORLESS, ct::COLORLESS, ct::COLORLESS] }], probe: false }) },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Energy, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Attach), EventPred::This(Role::Card)])), steps: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "IGNITION_ENERGY_MARKER", source: RuleSource::Energy }))] },
        Trigger { origin: RuleSource::Energy, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Checkup), EventPred::Owner(Who::Me)])), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::HasMarker { who: Who::Me, name: "IGNITION_ENERGY_MARKER", from: MarkerFrom::This }, Cond::AnySlot(SlotSel::One(SlotExpr::This), SlotPred::Any)]), yes: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::This, ..DiscardSpec::DEFAULT })), Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "IGNITION_ENERGY_MARKER", from: MarkerFrom::This }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
