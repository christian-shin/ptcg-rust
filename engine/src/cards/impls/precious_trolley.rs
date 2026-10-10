//! Precious Trolley (SSP, ACE SPEC): search your deck for any number of Basic
//! Pokémon and put them onto your Bench. Then, shuffle your deck.
//!
//! Twinleaf: the card moves hand→supporter and the TrainerEffect is
//! prevented before the empty-deck / full-bench checks (so a failed play
//! still throws after the move, which the rollback undoes). The prompt takes
//! 0..open-slots Basic Pokémon (no cancel); each goes to the next empty slot
//! via PlayPokemonFromDeckEffect, then the card moves supporter→discard and
//! a bare ShuffleDeckPrompt (no wait) follows.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PreciousTrolley",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::Basic]), bounds: Bounds { min: Num::Lit(0), max: Num::OpenBench(Who::Me) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
