//! Celebi (MEG 12): Traverse Time — search your deck for up to 3 in any
//! combination of [G] Pokémon and Stadium cards, reveal them, and put them
//! into your hand; then shuffle. Solar Cutter — 30.
//!
//! Twinleaf: SEARCH_DECK_FOR_CARDS_TO_HAND with an empty filter and every
//! other card blocked. Fixed in phase 4b (R4): the cards are revealed (the
//! prefab only revealed with a non-empty filter; Celebi now passes
//! `reveal = true`).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Celebi",
    // Traverse Time: search your deck for up to 3 in any combination of [G] Pokémon and Stadium
    // cards, reveal them, and put them into your hand; then shuffle.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(DECK, Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec {
                        predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::PrintedType(ct::GRASS)]), Pred::Stadium]),
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                        ..PickSpec::DEFAULT
                    },
                    destination: SearchDestination::Hand { reveal: true },
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
