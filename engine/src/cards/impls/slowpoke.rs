//! Slowpoke (SCR): Dangle Tail — put a Pokémon from your discard pile into
//! your hand.
//!
//! Fixed (W1-E): the chosen card is moved from the discard pile itself
//! (Twinleaf moved it from a fresh CardList, so it stayed in the discard pile
//! as well as going to the hand). Phase 4b (R4, Meta-Rulings): the chosen cards
//! are revealed to the opponent before they are moved (cards moving from the
//! discard pile to the hand are revealed). Phase 4b (R7E, ruling 1790): an attack
//! can be used even if its effect can't be carried out, so with no Pokémon in the
//! discard pile it is usable and does nothing (it used to be unusable).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Slowpoke@SCR",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Search(SearchSpec {
            pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
            destination: SearchDestination::Hand { reveal: true },
            msg: "CHOOSE_CARD_TO_HAND",
            cancel: false,
            shuffle_first: false,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
