//! Tremendous Bomb (PBL 82, Pokémon Tool): if the Pokémon this card is attached to isn't a Mega Evolution Pokémon ex, is in the
//! Active Spot, and takes 240 or more damage from an attack from your opponent's Mega Evolution Pokémon ex (even if this Pokémon
//! is Knocked Out), place 12 damage counters on the Attacking Pokémon. If you placed any damage counters in this way, discard
//! this card.
//!
//! Deluxe Bomb's trigger over the Damage event of an opponent's attack on the Active Pokémon holding the Tool (`DamagedActive`,
//! id1992), with three more conditions: the holder doesn't carry the "Mega ex" tag, the damage done (the final amount: the same
//! amount the Damage event carries, `EventPred::AmountAtLeast`) is 240 or more, and the Attacking Pokémon carries the "Mega ex"
//! tag. 12 counters on the Attacking Pokémon (a PlaceCounters event, not damage), then "if you do" (APR E-20) this Tool is
//! discarded; the Knock Out and the Prize cards wait for the effect to resolve (id2028).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TremendousBomb",
    triggers: &[Trigger {
        origin: RuleSource::Tool,
        event: Event::On(EventPred::All(&[
            DAMAGE_BY_OPP_ATTACKS,
            EventPred::Slot(SlotPred::All(&[SlotPred::Holder, SlotPred::Not(&SlotPred::Tag(tag::POKEMON_SV_MEGA))])),
            EventPred::DamagedActive,
            EventPred::AmountAtLeast(240),
            EventPred::Cause(CausePred::Pokemon(SlotPred::Tag(tag::POKEMON_SV_MEGA))),
        ])),
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
