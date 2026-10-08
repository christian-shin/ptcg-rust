//! Froslass (TWM): Freezing Shroud — during Pokémon Checkup, put 1 damage
//! counter on each Pokémon in play that has any Abilities (excluding any
//! Froslass). Frost Smash — 60.
//!
//! Twinleaf: at every EndTurnEffect, each Froslass card (wherever it is)
//! adds CHILLING_CURTAIN_MARKER (sourced by itself) to each player with an
//! in-play Froslass that has the ability, unless that player already has
//! the marker; at BetweenTurns for a player carrying the marker from this
//! card, it places 10 × (that player's Freezing Shroud Froslass count) on
//! every non-Froslass Pokémon with an Ability on both sides (the player's
//! first), then removes the marker. The ability lock check uses the marker
//! owner.
//!
//! Twinleaf fix (phase 4b, Y2-1): the Froslass count compares the ability name
//! with the literal 'Freezing Shroud'; it read `this.powers[0].name`, which
//! threw at every EndTurn for a copycat without Abilities that had copied
//! Frost Smash (Zoroark's Foul Play) while a Froslass was in play.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Froslass",
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Any }), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Cmp(Num::InPlayCount(Who::Me, PlayScope::All, Pred::All(&[Pred::Name("Froslass"), Pred::HasAbilityNamed("Freezing Shroud")])), CmpOp::Gt, Num::Lit(0)), Cond::Not(&Cond::HasMarker { who: Who::Me, name: "CHILLING_CURTAIN_MARKER", from: MarkerFrom::Any })]), yes: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "CHILLING_CURTAIN_MARKER", source: RuleSource::Ability }))], no: &[] })), Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Cmp(Num::InPlayCount(Who::Opp, PlayScope::All, Pred::All(&[Pred::Name("Froslass"), Pred::HasAbilityNamed("Freezing Shroud")])), CmpOp::Gt, Num::Lit(0)), Cond::Not(&Cond::HasMarker { who: Who::Opp, name: "CHILLING_CURTAIN_MARKER", from: MarkerFrom::Any })]), yes: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Opp), name: "CHILLING_CURTAIN_MARKER", source: RuleSource::Ability }))], no: &[] }))] },
        Trigger { origin: RuleSource::Ability, event: Event::OnCheckup(OnCheckupSpec {}), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::HasMarker { who: Who::Me, name: "CHILLING_CURTAIN_MARKER", from: MarkerFrom::This }, Cond::Not(&Cond::AbilityBlocked)]), yes: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Each(SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Not(&SlotPred::Named("Froslass")), SlotPred::HasAbility]))), counters: Num::InPlayCount(Who::Me, PlayScope::All, Pred::All(&[Pred::Name("Froslass"), Pred::HasAbilityNamed("Freezing Shroud")])), cause: CounterCause::Effect })), Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Each(SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::All(&[SlotPred::Not(&SlotPred::Named("Froslass")), SlotPred::HasAbility]))), counters: Num::InPlayCount(Who::Me, PlayScope::All, Pred::All(&[Pred::Name("Froslass"), Pred::HasAbilityNamed("Freezing Shroud")])), cause: CounterCause::Effect })), Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "CHILLING_CURTAIN_MARKER", from: MarkerFrom::This }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
