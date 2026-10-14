//! Gimmighoul (SSP 97): Minor Errand-Running - search your deck for up to 2 Basic Energy cards, reveal them, put them into your
//! hand. Then, shuffle your deck. Tackle - 50.
//!
//! The printed order: the search (revealed, into the hand), then one shuffle after it (APR E-19, D9). With an empty deck the
//! attack just does its damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Gimmighoul",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::BasicEnergy,
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                    ..PickSpec::DEFAULT
                },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
