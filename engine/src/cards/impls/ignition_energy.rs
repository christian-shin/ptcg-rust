//! Ignition Energy (SV11W, Special): provides [C]; [C][C][C] on an Evolution
//! Pokémon. If attached to 1 of your Pokémon, discard it at the end of your
//! turn.
//!
//! Twinleaf: attaching (AttachEnergyEffect of this card) adds a player
//! marker sourced by this card; between turns, for the player holding it,
//! every in-play slot holding the card discards it and removes the marker
//! (the marker stays if the card is no longer in play). No Special Energy
//! block probe anywhere. Stage is read from the slot's top Pokémon; not
//! Basic and not Restored gives [C][C][C].
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "IgnitionEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry { when: SlotPred::Basic, provides: &[ct::COLORLESS] }, ProvidedEntry { when: SlotPred::All(&[SlotPred::Top(Pred::Pokemon), SlotPred::Not(&SlotPred::Basic), SlotPred::Not(&SlotPred::StageIs(Stage::Restored))]), provides: &[ct::COLORLESS, ct::COLORLESS, ct::COLORLESS] }], probe: false }) },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Energy, event: Event::OnAttach(OnAttachSpec {}), steps: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "IGNITION_ENERGY_MARKER", source: RuleSource::Energy }))] },
        Trigger { origin: RuleSource::Energy, event: Event::OnCheckup(OnCheckupSpec {}), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::HasMarker { who: Who::Me, name: "IGNITION_ENERGY_MARKER", from: MarkerFrom::This }, Cond::AnySlot(SlotSel::One(SlotExpr::This), SlotPred::Any)]), yes: &[Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::This, ..MoveSpec::DEFAULT })), Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "IGNITION_ENERGY_MARKER", from: MarkerFrom::This }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
