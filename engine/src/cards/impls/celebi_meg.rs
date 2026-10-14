//! Celebi (MEG 12): Traverse Time - search your deck for up to 3 in any combination of [G] Pokémon and Stadium cards,
//! reveal them, and put them into your hand; then shuffle. Solar Cutter - 30.
//!
//! An attack's "up to 3" may be 0 (id2301). The cards are revealed and put into the hand (a PutIntoHand event, cause: this
//! attack), then the deck is shuffled once (APR E-19). With an empty deck there is nothing to search.
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
                        predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::GRASS)]), Pred::Stadium]),
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                        ..PickSpec::DEFAULT
                    },
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
