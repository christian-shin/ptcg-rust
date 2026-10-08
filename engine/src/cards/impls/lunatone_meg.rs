//! Lunatone (M1L): Lunar Cycle — once during your turn, if you have Solrock
//! in play, you may discard a Basic [F] Energy card from your hand in order
//! to use this Ability. Draw 3 cards (1 Lunar Cycle per turn). Power Gem — 50.
//!
//! Twinleaf keeps the flag on the player (`usedLunarCycle`, set only after a
//! discard); every copy resets it at the end of its owner's turn. Solrock may
//! be Active or Benched (matched by name). Cancelling the discard prompt
//! uses nothing.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Lunatone@Lunatone M1L|Lunatone ASC",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Not(&Cond::HasMarker { who: Who::Me, name: "LUNAR_CYCLE_MARKER", from: MarkerFrom::Any }), Cond::InPlay(Who::Me, PlayScope::All, Pred::Name("Solrock")), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]))],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, cancel: true, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::If(IfSpec { cond: Cond::Chosen(0), yes: &[Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })), Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) })), Step::new(Op::UseAbility(UseAbilitySpec { marker: "LUNAR_CYCLE_MARKER" }))], no: &[] })),
        ],
    }],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }), steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "LUNAR_CYCLE_MARKER", from: MarkerFrom::Any }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
