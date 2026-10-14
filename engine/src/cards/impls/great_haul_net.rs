//! Great Haul Net / Big Catch Net (CRI 78): choose 1 or both: shuffle up to 3 [W] Pokémon from your discard pile into your
//! deck; shuffle up to 3 Basic [W] Energy cards from your discard pile into your deck.
//!
//! One pick of 1 to 6 cards over the discard pile (public zone: at least 1 when able, id2199), at most 3 Pokémon and at most 3 Basic
//! Energy; they go into the deck, then one shuffle (APR E-19).
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
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
