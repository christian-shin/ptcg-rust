//! Shaymin (POR 3): Send Flowers — search your deck for an Energy card and attach it to 1 of your Benched [G] Pokémon; then
//! shuffle your deck. Leaf Step — 30.
//!
//! Any Energy card, a Special Energy card too (id2318). The Pokémon is chosen among the Benched Pokémon that are [G] as the game
//! checks it (APR D-07); the deck is shuffled after the search, whatever was found (APR E-19).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "ShayminPOR",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(DECK, Pred::Any),
            yes: &[
                Step::new(Op::Attach(AttachSpec {
                    from: DECK,
                    predicate: Pred::Energy,
                    slots: AttachSlots::Bench,
                    target: SlotPred::TypeIs(ct::GRASS),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                    ..AttachSpec::DEFAULT
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
