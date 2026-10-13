//! Eelektrik (ASC 60 / SV11B): Dynamotor — once during your turn (before your attack), you may attach a [L] Energy card from your
//! discard pile to 1 of your Benched Pokémon. Electric Ball — 50.
//!
//! An Ability: one Attach from the discard pile to a Benched Pokémon, a Basic Lightning Energy card, a real choice (it can be
//! cancelled), exactly one card. It counts as used only when a card was attached (`Cond::Attached`): then a player marker
//! "from This" is set, so each Eelektrik uses its own Dynamotor (JP FAQ audit: two Eelektrik each use Dynamotor), and an
//! EndTurn trigger clears it.
use crate::spec::prelude::*;

const DYNAMOTOR: &str = "DYNAMOTOR_MAREKER";

pub static SPEC: CardSpec = CardSpec {
    class: "Eelektrik@ASC",
    // Dynamotor: once during your turn, you may attach a [L] Energy card from your discard pile to
    // 1 of your Benched Pokémon (it counts as used only when an Energy was attached).
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Not(&Cond::HasMarker { who: Who::Me, name: DYNAMOTOR, from: MarkerFrom::This })],
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]),
                slots: AttachSlots::Bench,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                cancel: true,
                ..AttachSpec::DEFAULT
            })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Attached,
                yes: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: DYNAMOTOR, source: RuleSource::Ability }))],
                no: &[],
            })),
        ],
    }],
    triggers: &[Trigger {
        origin: RuleSource::CardRule,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EndTurn), EventPred::Owner(Who::Me)])),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: DYNAMOTOR, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
