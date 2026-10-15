//! Risky Ruins (MEG, class key DangerousRuins): whenever any player puts a Basic non-[D] Pokémon onto their Bench during their
//! turn, place 2 damage counters on that Pokémon.
//!
//! A Stadium trigger over the EnterPlay event: a Basic non-[D] Pokémon entering a Bench spot of its owner during the owner's
//! turn, whatever the source (hand, deck, discard pile: id2233; a Fossil: id2429), after it is on the Bench
//! (docs/rulings/RULES.md). Retreating or switching isn't putting onto the Bench (id2329), nor is devolving (id2328): neither is
//! EnterPlay.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RiskyRuins",
    passives: &[
        // Automatically active: it can't be announced and used.
    ],
    // Whenever any player puts a Basic non-[D] Pokémon onto their Bench during their turn, place 2 damage
    // counters on that Pokémon.
    triggers: &[Trigger {
        origin: RuleSource::Stadium,
        event: Event::On(EventPred::All(&[
            EventPred::Kind(EventKind::EnterPlay),
            EventPred::Slot(SlotPred::IsBench),
            EventPred::Card(Pred::All(&[Pred::Basic, Pred::Not(&Pred::PokemonType(ct::DARK))])),
            EventPred::Turn(TurnOf::EventOwner),
        ])),
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(2) }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
