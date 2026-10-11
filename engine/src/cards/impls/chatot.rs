//! Chatot (TEF 132): A Cappella: search your deck for up to 3 Basic Pokémon and put them onto your Bench. Then, shuffle your
//! deck. Gust: 20.
//!
//! With no empty Bench spot the attack ends without a search and without a shuffle (APR C-11, id362: the whole effect fails);
//! an empty deck can't be searched (id840). Otherwise the chosen Pokémon enter play from the deck (EnterPlay events, this
//! attack their cause) and the deck is shuffled (APR E-19).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Chatot",
    // A Capella: search your deck for up to 3 Basic Pokémon and put them onto your Bench, then
    // shuffle (nothing happens with no empty Bench space).
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::BenchSpace(Who::Me),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Bench,
                    msg: "",
                    cancel: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
