//! Charcadet (PFL): Gather Strength — search your deck for up to 2 Basic Energy cards, reveal them, put them into your hand.
//! Then, shuffle your deck. Chop — 10.
//!
//! With an empty deck the attack does nothing: a deck known to be empty can't be searched (id840, id362). Otherwise the
//! Search puts up to 2 cards into the hand (a PutIntoHand of the attack's effect, revealed, none required: a search for a
//! kind) and the shuffle follows (APR E-19).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "CharcadetPFL",
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
