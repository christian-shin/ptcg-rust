//! Dwebble (DRI): Ascension — search your deck for a card that evolves from
//! this Pokémon and put it onto this Pokémon to evolve it, then shuffle.
//!
//! Twinleaf: an empty deck skips everything (no shuffle); the evolution is a
//! plain MOVE_CARDS onto the player's Active followed by `clearEffects()` and
//! `pokemonPlayedTurn = turn` (no EvolveEffect); the shuffle has no wait.
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
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
