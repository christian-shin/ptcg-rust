//! Fighting Gong (M1L, item): search your deck for a Basic [F] Pokémon or a
//! Basic [F] Energy card, reveal it, and put it into your hand; then shuffle.
//!
//! Twinleaf quirks kept: the item is moved to the supporter pile (and never
//! discarded by the card); the energy count is passed as `maxTrainers`, so
//! `max = min(pokemon,1) || min(energy,1)`; ShowCards only when something
//! was taken.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);
const POKEMON: Pred = Pred::All(&[Pred::Pokemon, Pred::Basic, Pred::PokemonType(crate::types::ct::FIGHTING)]);
const ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]);

pub static SPEC: CardSpec = CardSpec {
    class: "FightingGong",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    from: DECK,
                    predicate: Pred::OneOf(&[POKEMON, ENERGY]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::CardCount(DECK, Pred::OneOf(&[POKEMON, ENERGY])), &Num::Lit(1)) },
                    caps: &[
                        Cap { kind: CapKind::Pokemon, max: Num::Min(&Num::CardCount(DECK, POKEMON), &Num::Lit(1)) },
                        Cap { kind: CapKind::Trainer, max: Num::Min(&Num::CardCount(DECK, ENERGY), &Num::Lit(1)) },
                    ],
                    ..PickSpec::DEFAULT
                },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
