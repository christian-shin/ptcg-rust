//! Fighting Gong (M1L, item): search your deck for a Basic [F] Pokémon or a
//! Basic [F] Energy card, reveal it, and put it into your hand; then shuffle.
//!
//! Twinleaf quirks kept: the item is moved to the supporter pile (and never
//! discarded by the card); the energy count is passed as `maxTrainers`, so
//! `max = min(pokemon,1) || min(energy,1)`; ShowCards only when something
//! was taken.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FightingGong",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::Basic, Pred::PokemonType(crate::types::ct::FIGHTING)]), Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")])]), bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::Basic, Pred::PokemonType(crate::types::ct::FIGHTING)]), Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")])])), &Num::Lit(1)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
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
