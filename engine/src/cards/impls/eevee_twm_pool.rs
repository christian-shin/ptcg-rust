//! Eevee (TWM 135): Ascension — search your deck for a card that evolves from this Pokémon and put it onto this Pokémon to evolve
//! it. Then, shuffle your deck. Quick Attack — 20+; flip a coin, if heads 20 more damage.
//!
//! An empty deck can't be searched (id840): nothing happens, no shuffle. Otherwise one pick (min 0: a search for a kind may
//! find nothing) of a Pokémon that evolves from Eevee, an Evolve event from the deck onto this Pokémon (not played from the
//! hand: id1133), then one shuffle after the search (APR E-19); with no such card in the deck only the shuffle happens.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);
const EVOLUTION: Pred = Pred::All(&[Pred::Pokemon, Pred::EvolvesFromActive]);

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
