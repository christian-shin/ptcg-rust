//! Misty's Vitality (PBL 80, Supporter): search your deck for up to 4 Basic [W] Energy cards and attach them to 1 of your
//! Pokémon. Then, shuffle your deck. Your turn ends.
//!
//! One search of 0 to 4 Basic [W] Energy cards attached to one Pokémon of yours (the Pokémon is picked after the cards),
//! the shuffle after it (APR E-19), then the end of the turn as the last step (it ends the turn even when nothing was found:
//! id1963, Boxed Order). With an empty deck the card can't be played (id925).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "MistysVitality",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: DECK, predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Water Energy")]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(4) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::AttachToPicked,
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            Step::new(Op::EndTurn(EndTurnSpec { who: Who::Me })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
