//! Spiky Energy (JTG): provides [C]. If the Pokémon this card is attached to
//! is in the Active Spot and is damaged by an attack from your opponent's
//! Pokémon (even if it is Knocked Out), put 2 damage counters on the
//! Attacking Pokémon.
//!
//! An `Event::On` trigger over the Damage event of an opponent's attack on the Active Pokémon this card is attached to
//! (`DamagedActive`, the Active Spot read when the damage is done, id1992): Damage is made only for more than 0 that no
//! prevention stopped, so damage that was prevented (Crustle's Mysterious Rock Inn) or reduced to 0 doesn't count. It
//! stacks (one per copy, ruling 1646) and works wherever the Pokémon end up (ruling 1839).
//!
//! Step 7 of the attack flow chart (APR E-03): a trigger over the Damage an attack did applies after every effect of the
//! attack's own text and its prompts, in the attack's window (`run::fire_on`). It needs this card to be still attached to
//! the damaged Pokémon (an attack that discards it stops it, ruling 1649) and the Special Energy not blocked, and the
//! Attacking Pokémon still in play, wherever it is (rulings 530, 1839: `CausePokemon`). The counters are one PlaceCounters
//! event on the Attacking Pokémon, cause this card's Energy rule.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SpikyEnergy",
    // When the Pokémon this is attached to, in the Active Spot, is damaged by an attack from your opponent's Pokémon
    // (even if Knocked Out), put 2 damage counters on the Attacking Pokémon.
    triggers: &[Trigger {
        origin: RuleSource::Energy,
        event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::Holder), EventPred::DamagedActive])),
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::CausePokemon), counters: Num::Lit(2) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
