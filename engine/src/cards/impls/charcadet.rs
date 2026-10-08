//! Charcadet (M2 / PFL): Gather Power — search your deck for up to 2 Basic
//! Energy cards and put them into your hand. Chop — 10.
//!
//! Fixed (phase 4b): an empty deck does nothing (it threw CANNOT_USE_ATTACK,
//! making the attack unusable); the up-to-2 choice (cancel not allowed) is
//! moved to hand, revealed to the opponent, and the deck is shuffled (both
//! were missing).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Charcadet@Charcadet M2",
    // Gather Power: search your deck for up to 2 Basic Energy cards and put them into your hand
    // (revealed), then shuffle.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(DECK, Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { predicate: Pred::BasicEnergy, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: true },
                    msg: "",
                    cancel: false,
                    shuffle_first: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
