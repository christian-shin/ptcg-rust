//! Buddy-Buddy Poffin (TEF): search your deck for up to 2 Basic Pokémon with
//! 70 HP or less and put them onto your Bench. Then, shuffle your deck.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BuddyBuddyPoffin",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    chooser: Who::Me,
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::All(&[Pred::Basic, Pred::HpAtMost(70)]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                    ..PickSpec::DEFAULT
                },
                destination: SearchDestination::Bench,
                msg: "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
