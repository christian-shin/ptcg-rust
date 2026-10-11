//! Dwebble (DRI): Ascension — search your deck for a card that evolves from this Pokémon and put it onto this Pokémon to
//! evolve it. Then, shuffle your deck.
//!
//! An empty deck can't be searched (id840): nothing happens, no shuffle. The evolving is an Evolve event from the deck
//! (not played from the hand: id1133, id2037), caused by the attack; the choice can be declined (a search for a kind).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Dwebble",
    // Ascension: search your deck for a card that evolves from this Pokémon and put it onto this
    // Pokémon to evolve it, then shuffle.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(DECK, Pred::Any),
            yes: &[
                Step::new(Op::Pick(PickSpec {
                    from: DECK,
                    predicate: Pred::All(&[Pred::Pokemon, Pred::StageIs(crate::types::Stage::Stage1), Pred::EvolvesFrom("Dwebble")]),
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    into: 0,
                    cancel: true,
                    msg: "CHOOSE_CARD_TO_EVOLVE",
                    ..PickSpec::DEFAULT
                })),
                Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::PutOnto { slot: MY_ACTIVE, card: 0 } })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
