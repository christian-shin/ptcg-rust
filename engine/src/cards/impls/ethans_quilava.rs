//! Ethan's Quilava (DRI): Adventure Bound — once during your turn, you may
//! search your deck for an Ethan's Adventure, reveal it and put it into your
//! hand, then shuffle. Combustion — 40.
//!
//! Twinleaf: the marker check (BLOCKED_BY_EFFECT) and empty-deck check
//! (NO_CARDS_IN_DECK) come first (no ability-lock probe here); the choice is
//! cancellable (0-1); declining only shuffles. Taking the card shows it to the
//! opponent, moves it, adds the marker (player marker sourced by this card),
//! marks the ability used and shuffles.
use crate::spec::prelude::*;

const BOUND: &str = "ADVENTURE_BOUND";

pub static SPEC: CardSpec = CardSpec {
    class: "EthansQuilava",
    // Adventure Bound: once during your turn, you may search your deck for an Ethan's Adventure,
    // reveal it and put it into your hand, then shuffle (it counts as used when you take the card).
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Not(&Cond::HasMarker { who: Who::Me, name: BOUND, from: MarkerFrom::This })],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { predicate: Pred::Name("Ethan's Adventure"), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "CHOOSE_CARD_TO_HAND",
                cancel: true,
                shuffle_first: false,
            })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Chosen(0),
                yes: &[
                    Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: BOUND, source: RuleSource::Ability })),
                    Step::new(Op::AbilityUsed(AbilityUsedSpec {})),
                ],
                no: &[],
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: BOUND, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
