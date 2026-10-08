//! Boomerang Energy (TWM): provides [C]. If discarded by an effect of an
//! attack of the Pokémon it is attached to, attach it from the discard pile
//! to that Pokémon after attacking.
//!
//! Twinleaf quirk kept: the card is re-attached to whatever is Active then.
//!
//! Fixed (phase 4b, R7F-10; ruling 1650): it was re-attached at EndTurn; it is
//! now re-attached in AfterAttackTriggersEffect, once the attack's effects and the
//! Energy choices they ask after the damage are done (EndTurn stays as the fallback
//! for a discard no AfterAttackTriggersEffect followed), before the effects that trigger
//! on the Defending Pokémon (Handheld Fan) resolve: AfterAttackEffect reaches
//! Pokémon, then Energy, then Trainers.
//!
//! Fixed (phase 4b, W4): the re-attach was armed by a marker set on the
//! AttackEffect, but the attacker's own handler runs before its attached
//! Energy sees that effect, so a discard made synchronously by the attack
//! (Volt Strike, "discard all Energy") happened before the marker existed and
//! the card was never re-attached. It is now armed directly by a
//! DiscardCardsEffect from the Active holding this card that lists this card.
use crate::spec::prelude::*;

const DISCARDED: &str = "BOOMERANG_DISCARDED_MARKER";

/// Attach this card from the discard pile to the Active Pokémon, once, when it was discarded by
/// an effect of its Pokémon's attack.
const REATTACH: &[Step] = &[Step::new(Op::If(IfSpec {
    cond: Cond::HasMarker { who: Who::Me, name: DISCARDED, from: MarkerFrom::This },
    yes: &[
        Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: DISCARDED, from: MarkerFrom::This })),
        Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::This, place: Place::AttachTo(MY_ACTIVE), ..MoveSpec::DEFAULT })),
    ],
    no: &[],
}))];

pub static SPEC: CardSpec = CardSpec {
    class: "BoomerangEnergy",
    triggers: &[
        Trigger {
            origin: RuleSource::Energy,
            event: Event::OnDiscarded(OnDiscardedSpec {}),
            steps: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: DISCARDED, source: RuleSource::Energy }))],
        },
        // After the attack's effects (and the Energy choices they ask) are done; the end of the
        // turn is the fallback for a discard no after-attack window followed.
        Trigger { origin: RuleSource::Energy, event: Event::OnAfterAttackTriggers(OnAfterAttackTriggersSpec {}), steps: REATTACH },
        Trigger { origin: RuleSource::Energy, event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }), steps: REATTACH },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
