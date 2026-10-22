//! Bombirdier (PBL 71): Challenging Delivery - flip 2 coins. If both of them are heads, search your deck for a Pokémon and put it
//! onto your Bench. Then, shuffle your deck. Speed Wing - 100.
//!
//! The coins always flip; with 2 heads and room on the Bench, one search for a Pokémon (0 or 1: a search for a kind) onto the
//! Bench, then the shuffle (Swinub JTG's form, APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Bombirdier",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            flips: Flips::Count(Num::Lit(2)),
            then: &[Step::new(Op::If(IfSpec {
                cond: Cond::All(&[Cond::Cmp(Num::Heads, CmpOp::Eq, Num::Lit(2)), Cond::BenchSpace(Who::Me)]),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Bench,
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
