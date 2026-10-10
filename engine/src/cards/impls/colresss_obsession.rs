//! Colress's Tenacity (SFA): search your deck for a Stadium card and an
//! Energy card, reveal them, put them into your hand, shuffle.
//!
//! Twinleaf: every deck card that is neither a Stadium nor an Energy is
//! blocked; `max = min(stadiums,1) + min(energies,1)` with `maxTrainers` /
//! `maxEnergies`; ShowCards only when something was taken; the final shuffle
//! prompt has no wait.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "ColresssObsession",
    // Search your deck for a Stadium card and an Energy card, reveal them, put them into your
    // hand, shuffle.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    predicate: Pred::OneOf(&[Pred::Stadium, Pred::Energy]),
                    bounds: Bounds {
                        min: Num::Lit(0),
                        max: Num::Add(&Num::Min(&Num::CardCount(DECK, Pred::Stadium), &Num::Lit(1)), &Num::Min(&Num::CardCount(DECK, Pred::Energy), &Num::Lit(1))),
                    },
                    caps: &[
                        Cap { kind: CapKind::Trainer, max: Num::Min(&Num::CardCount(DECK, Pred::Stadium), &Num::Lit(1)) },
                        Cap { kind: CapKind::Energy, max: Num::Min(&Num::CardCount(DECK, Pred::Energy), &Num::Lit(1)) },
                    ],
                    ..PickSpec::DEFAULT
                },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
