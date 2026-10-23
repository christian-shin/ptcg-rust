//! Nitro [R] Energy (CRI 86, Special Energy; the official name is Nitro Fire Energy): as long as this card is attached to a
//! Pokémon, it provides [R] Energy. If this card is discarded by an effect of an attack used by the [R] Pokémon this card is
//! attached to, put this card into your hand after attack damage and effects.
//!
//! Boomerang Energy's shape with the other destination and the type condition: the trigger is its own LeavePlay to the discard
//! pile caused by an attack of the Pokémon it was attached to (`CauseOnSlot`, read on the spot it left) while that Pokémon is [R]
//! as the game checks its type (APR D-07; `CausePred::Pokemon(TypeIs)`), whatever route discards it. It records that Pokémon;
//! the card goes from the discard pile into the hand in the after-attack window (the `AttackDone` event of its owner's attack),
//! after the attack's effects and the Energy choices they ask, as Boomerang Energy re-attaches. The record is only the memory
//! that the discard happened (the Pokémon may have moved since); an attack that discards a card with another cause, or of a
//! Pokémon that isn't [R], leaves the card in the discard pile.
use crate::spec::prelude::*;

/// Put this card from the discard pile into the hand, once, when it was discarded by an effect of its Pokémon's attack.
const TO_HAND: &[Step] = &[Step::new(Op::If(IfSpec {
    cond: Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::Recorded),
    yes: &[
        Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::This, ..PutIntoHandSpec::DEFAULT })),
        Step::new(Op::Forget(ForgetSpec::Every(Who::Me))),
    ],
    no: &[],
}))];

pub static SPEC: CardSpec = CardSpec {
    class: "NitroFireEnergy",
    triggers: &[
        Trigger {
            origin: RuleSource::Energy,
            event: Event::On(EventPred::All(&[
                EventPred::Kind(EventKind::LeavePlay),
                EventPred::This(Role::Card),
                EventPred::Dest(RulesZone::Discard),
                EventPred::Cause(CausePred::All(&[CausePred::Kind(crate::cause::CauseKind::Attack), CausePred::Pokemon(SlotPred::TypeIs(ct::FIRE))])),
                EventPred::CauseOnSlot,
            ])),
            steps: &[Step::new(Op::Record(SlotExpr::Picked))],
        },
        Trigger { origin: RuleSource::Energy, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::AttackDone), EventPred::Owner(Who::Me)])), steps: TO_HAND },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
