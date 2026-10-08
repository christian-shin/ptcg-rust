//! Great Haul Net / Big Catch Net (CRI): choose 1 or both — shuffle up to 3
//! [W] Pokémon and/or up to 3 Basic [W] Energy cards from your discard pile
//! into your deck.
//!
//! Twinleaf: one ChooseCardsPrompt (min 1 since phase 4b, rulings 1778/1853: public zone; max 6, maxPokemons 3,
//! maxBasicEnergies 3) over the discard with the other cards blocked; then
//! the card moves from wherever it is to the discard before a wait-less
//! shuffle.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GreatHaulNet",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::PokemonType(crate::types::ct::WATER)]), Pred::All(&[Pred::BasicEnergy, Pred::Provides(crate::types::ct::WATER)])]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(6) }, caps: &[Cap { kind: CapKind::Pokemon, max: Num::Lit(3) }, Cap { kind: CapKind::BasicEnergy, max: Num::Lit(3) }], ..PickSpec::DEFAULT },
                destination: SearchDestination::Deck { reveal: false },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
