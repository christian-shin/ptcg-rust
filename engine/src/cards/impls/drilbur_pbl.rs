//! Drilbur (PBL / M5): Call for Family — search your deck for up to 2 Basic
//! Pokémon and put them onto your Bench, then shuffle. Dig Claws — 50.
//!
//! SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_ONTO_BENCH({ stage: BASIC },
//! { min: 0, max: 2 }): throws on an empty deck or a full Bench, except during
//! an attack (phase 4b R7E, rulings 336/337/1790: the attack is usable, the
//! search just fails; the prefab returns without searching, in both engines).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Drilbur@PBL",
    // Call for Family: search your deck for up to 2 Basic Pokémon and put them onto your Bench,
    // then shuffle (the attack just does its damage with an empty deck or a full Bench).
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::BenchSpace(Who::Me)]),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Bench,
                    msg: "",
                    cancel: true,
                    shuffle_first: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
