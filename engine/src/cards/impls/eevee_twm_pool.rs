//! Eevee (TWM 135): Ascension — search your deck for a card that evolves from
//! this Pokémon and put it onto this Pokémon to evolve it, then shuffle.
//! Quick Attack — 20+; flip a coin, if heads 20 more damage.
//!
//! Twinleaf: an empty deck skips everything; with no evolution in the deck only
//! SHUFFLE_DECK runs; else a non-cancellable ChooseCardsPrompt ({ superType:
//! POKEMON }, min 0, max 1) with every non-evolution index blocked. The
//! evolution is a plain MOVE_CARDS onto the slot holding this card, then
//! `clearEffects()`, `pokemonPlayedTurn = turn`, SHUFFLE_DECK.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);
const EVOLUTION: Pred = Pred::All(&[Pred::Pokemon, Pred::EvolvesFrom("Eevee")]);

pub static SPEC: CardSpec = CardSpec {
    class: "EeveeTWMPool",
    attacks: &[
        // Ascension: search your deck for a card that evolves from this Pokémon and put it onto this
        // Pokémon to evolve it, then shuffle.
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(DECK, Pred::Any),
                yes: &[
                    Step::new(Op::If(IfSpec {
                        cond: Cond::Nonempty(DECK, EVOLUTION),
                        yes: &[
                            Step::new(Op::Pick(PickSpec {
                                from: DECK,
                                predicate: EVOLUTION,
                                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                                into: 0,
                                msg: "CHOOSE_CARD_TO_EVOLVE",
                                ..PickSpec::DEFAULT
                            })),
                            Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::PutOnto { slot: SlotExpr::This, card: 0 } })),
                        ],
                        no: &[],
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
                ],
                no: &[],
            }))],
        },
        // Quick Attack: flip a coin, if heads 20 more damage.
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(20, Cond::True))], ..CoinSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
