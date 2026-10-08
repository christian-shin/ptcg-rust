//! Exeggcute (SSP 1): Precocious Evolution (usable on the first turn) —
//! search your deck for a card that evolves from this Pokémon and put it
//! onto this Pokémon to evolve it, then shuffle.
//!
//! Two `Exeggcute` classes exist; this port is bound to SSP. Twinleaf: a
//! non-cancellable ChooseCardsPrompt (min 0, max 1); an empty deck skips everything (no shuffle); the evolution is a
//! plain MOVE_CARDS onto the player's Active followed by `clearEffects()` and
//! `pokemonPlayedTurn = turn` (no EvolveEffect); the shuffle has no wait.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Exeggcute@SSP",
    // Precocious Evolution: search your deck for a card that evolves from this Pokémon and put it
    // onto this Pokémon to evolve it, then shuffle.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(DECK, Pred::Any),
            yes: &[
                Step::new(Op::Pick(PickSpec {
                    from: DECK,
                    predicate: Pred::All(&[Pred::Pokemon, Pred::EvolvesFrom("Exeggcute")]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                    into: 0,
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
