//! Eelektrik (SV11B): Dynamotor — once during your turn, you may attach a
//! [L] Energy card from your discard pile to 1 of your Benched Pokémon.
//! Electric Ball — 50.
//!
//! Twinleaf: throws CANNOT_USE_POWER without a Benched Pokémon or a basic
//! Energy providing [L] in the discard, POWER_ALREADY_USED with the marker;
//! then a cancellable AttachEnergyPrompt (discard → Bench, basic 'Lightning
//! Energy', min 1 max 1). The marker is set only when a transfer is made
//! (no ABILITY_USED board effect). PlayPokemonEffect of this card and every
//! EndTurnEffect clear the marker.
use crate::spec::prelude::*;

const DYNAMOTOR: &str = "DYNAMOTOR_MAREKER";

pub static SPEC: CardSpec = CardSpec {
    class: "Eelektrik@BLK",
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
        origin: RuleSource::Ability,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: DYNAMOTOR, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
