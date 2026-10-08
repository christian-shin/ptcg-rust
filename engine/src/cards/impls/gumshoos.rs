//! Gumshoos (M1L): Gather Evidence — once during your turn, switch a card
//! from your hand with the top card of your deck. Bite — 50.
//!
//! Twinleaf: CANNOT_USE_POWER with an empty deck or hand, POWER_ALREADY_USED
//! with the marker, then a cancellable ChooseCardsPrompt on the hand. On a
//! pick: MOVE_CARDS deck→hand (count 1), then the picked card is spliced out
//! of the hand and unshifted onto the deck directly, marker + ABILITY_USED.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Gumshoos",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Not(&Cond::HasMarker { who: Who::Me, name: "GATHER_EVIDENCE_MARKER", from: MarkerFrom::This }), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::Any)],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, cancel: true, into: 0, msg: "CHOOSE_CARD_TO_DECK", ..PickSpec::DEFAULT })),
            Step::new(Op::If(IfSpec { cond: Cond::Chosen(0), yes: &[Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Deck), to: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Top(Num::Lit(1)), ..MoveSpec::DEFAULT })), Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Chosen(0), place: Place::Top, ..MoveSpec::DEFAULT })), Step::new(Op::UseAbility(UseAbilitySpec { marker: "GATHER_EVIDENCE_MARKER" }))], no: &[] })),
        ],
    }],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }), steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "GATHER_EVIDENCE_MARKER", from: MarkerFrom::This }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
