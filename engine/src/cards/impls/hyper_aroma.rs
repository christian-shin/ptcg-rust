//! Hyper Aroma (TWM 152, ACE SPEC): search your deck for up to 3 Stage 1 Pokémon, reveal them, and put them into your
//! hand. Then, shuffle your deck.
//!
//! The chosen cards are revealed before they move (id1131), go into the hand (PutIntoHand, this Item its cause), and the
//! deck is shuffled after the search (APR E-19).
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "HyperAroma",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::StageIs(Stage::Stage1), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
