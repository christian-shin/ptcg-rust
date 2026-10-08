//! Team Rocket's Proton (DRI): usable on the first turn if you go first.
//! Search your deck for up to 3 Basic Team Rocket's Pokémon, reveal them,
//! put them into your hand, then shuffle.
//!
//! Twinleaf: sets `rocketSupporter`; the prompt message is
//! CHOOSE_CARD_TO_PUT_ONTO_BENCH; non-Team Rocket cards are blocked (by
//! first index), the filter is Basic Pokémon.
//!
//! R7C: `rocket_supporter` is not set when used as the effect of an attack (ruling 1727).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsProton",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::All(&[Pred::Basic, Pred::Tag(tag::TEAM_ROCKET)]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                    ..PickSpec::DEFAULT
                },
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
