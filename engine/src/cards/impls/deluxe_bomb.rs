//! Deluxe Bomb (SCR 134, Pokémon Tool, ACE SPEC): if the Pokémon this card is attached to is in the Active Spot and is damaged by
//! an attack from your opponent's Pokémon (even if this Pokémon is Knocked Out), put 12 damage counters on the Attacking Pokémon.
//! If you placed any damage counters in this way, discard this card.
//!
//! An `Event::On` trigger over the Damage event of an opponent's attack on the Active Pokémon holding the Tool (as Punk Helmet
//! PFL 92), resolved in the E-03 window: 12 counters as a PlaceCounters event on the Attacking Pokémon (nothing when it left
//! play), then, "if you do" (APR E-20), this Tool is discarded. The Knock Out and the Prize cards wait for the effect to resolve
//! (id2028).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DeluxeBomb",
    triggers: &[Trigger {
        origin: RuleSource::Tool,
        event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::Holder), EventPred::DamagedActive])),
        steps: &[
            Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::CausePokemon), counters: Num::Lit(12) })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Done,
                yes: &[Step::new(Op::Discard(DiscardSpec { cards: CardSel::This, ..DiscardSpec::DEFAULT }))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
