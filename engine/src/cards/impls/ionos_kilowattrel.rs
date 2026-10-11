//! Iono's Kilowattrel (ASC 72): Flashing Draw - you must discard a Basic [L] Energy from this Pokémon in order to use
//! this Ability; once during your turn, you may draw cards until you have 6 cards in your hand. Mach Bolt - 70.
//!
//! The Energy is the cost (`Step::cost`, APR E-30): a LeavePlay (D1) caused by this Ability, and the Ability can't be used
//! without one. The `needs` entries (fewer than 6 cards in hand, a card in the deck) restate that an Ability that can't
//! have any effect can't be used (id2108) and go at the D10 switch; the once-per-turn marker is cleared at the end of the
//! owner's turns. The draw is one Draw event of the cards missing from 6.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IonosKilowattrel",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Not(&Cond::HasMarker { who: Who::Me, name: "RUMBLING_ENGINE_MARKER", from: MarkerFrom::This }), Cond::AnySlot(SlotSel::One(SlotExpr::This), SlotPred::HasCard(Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")])))],
        steps: &[
            Step::cost(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::This), selection: EnergySelection::CostOne { pred: Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]) }, ..DiscardEnergySpec::DEFAULT })),
            Step::new(Op::If(IfSpec { cond: Cond::Cmp(Num::Last, CmpOp::Gt, Num::Lit(0)), yes: &[Step::new(Op::AbilityUsed(AbilityUsedSpec { marker: Some("RUMBLING_ENGINE_MARKER") })), Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))], no: &[] })),
        ],
    }],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }), steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "RUMBLING_ENGINE_MARKER", from: MarkerFrom::This }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
