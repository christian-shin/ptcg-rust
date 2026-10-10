//! Hilda (SV11W / WHT): search your deck for an Evolution Pokémon and an
//! Energy card, reveal them, put them into your hand, then shuffle.
//!
//! Twinleaf: every non-(Evolution Pokémon / Energy) deck card is blocked,
//! `max = min(evolutions,1) + min(energies,1)` with `maxPokemons` /
//! `maxEnergies`; ShowCards only when something was taken; the final
//! ShuffleDeckPrompt has no trailing wait.
//!
//! Fixed (phase 4b, R2): playable with an empty deck; it now throws
//! CANNOT_PLAY_THIS_CARD before any state change (rulings 779, 851).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hilda",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)]), Pred::Energy]), bounds: Bounds { min: Num::Lit(0), max: Num::Add(&Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)])), &Num::Lit(1)), &Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Energy), &Num::Lit(1))) }, caps: &[Cap { kind: CapKind::Pokemon, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)])), &Num::Lit(1)) }, Cap { kind: CapKind::Energy, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Energy), &Num::Lit(1)) }], ..PickSpec::DEFAULT },
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
