//! Team Rocket's Proton (DRI): if you go first, you may use this card during your first turn. Search your deck for up to 3
//! Basic Team Rocket's Pokémon, reveal them, and put them into your hand. Then, shuffle your deck.
//!
//! The first-turn permission is the card's `first_turn` flag (read by the play rule). One Search op revealing what it finds, then one
//! shuffle after it (APR E-19). A Team Rocket's Supporter played from the hand is read from the turn's play record
//! (`Cond::PlayedThisTurn`); used through Look-Alike Show it isn't played (D5).
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
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
