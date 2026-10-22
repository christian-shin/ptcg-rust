//! Team Rocket's Koffing (DRI 125): Smog Signals - if this Pokémon is in the Active Spot and is damaged by an attack from your
//! opponent's Pokémon (even if this Pokémon is Knocked Out), search your deck for up to 2 Pokémon that have "Koffing" in their name and
//! put them onto your Bench. Then, shuffle your deck. Leaking Gas - 30.
//!
//! A trigger over the Damage event of an opponent's attack on this Pokémon while it is the Active Pokémon (`DamagedActive`, id1992),
//! resolved after the attack's own effects (id2172): with room on the Bench the search is made (the Ability is not optional,
//! id2171: you search and shuffle, finding none if you like; 0 to 2 Pokémon whose name contains "Koffing"), even if this Pokémon is
//! the only one and is Knocked Out (id2153: the Pokémon go onto the Bench before the Knock Out check).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsKoffing",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::Holder), EventPred::DamagedActive])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::BenchSpace(Who::Me),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec {
                        from: ZoneRef(Who::Me, Zone::Deck),
                        predicate: Pred::All(&[Pred::Pokemon, Pred::NameContains("Koffing")]),
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                        ..PickSpec::DEFAULT
                    },
                    destination: SearchDestination::Bench,
                    cancel: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
