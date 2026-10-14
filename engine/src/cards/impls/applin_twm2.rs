//! Applin (TWM 126): Find a Friend: search your deck for a Pokémon, reveal it, and put it into your hand. Then, shuffle
//! your deck. Rolling Tackle: 30.
//!
//! The search shows the chosen card to the opponent before it moves (id1131) and puts it into the hand (a PutIntoHand event,
//! this attack its cause); the shuffle follows (APR E-19). An empty deck can't be searched (id840): the attack then does
//! nothing, with no search and no shuffle.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Applin@TWM",
    // Find a Friend: search your deck for a Pokémon, reveal it, put it into your hand, then shuffle.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { predicate: Pred::Pokemon, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: true },
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
