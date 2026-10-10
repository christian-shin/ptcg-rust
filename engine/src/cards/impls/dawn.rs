//! Dawn (M2 / PFL): search your deck for a Basic Pokémon, a Stage 1 Pokémon
//! and a Stage 2 Pokémon, reveal them, put them into your hand, then shuffle.
//!
//! Twinleaf: non-matching deck cards are blocked, `max` is the number of
//! stages present, with `maxBasics` / `maxStage1` / `maxStage2`; ShowCards
//! only when something was taken; no trailing wait after the shuffle.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);
const BASICS: Num = Num::Min(&Num::CardCount(DECK, Pred::Basic), &Num::Lit(1));
const STAGE1: Num = Num::Min(&Num::CardCount(DECK, Pred::All(&[Pred::Pokemon, Pred::StageIs(crate::types::Stage::Stage1)])), &Num::Lit(1));
const STAGE2: Num = Num::Min(&Num::CardCount(DECK, Pred::All(&[Pred::Pokemon, Pred::StageIs(crate::types::Stage::Stage2)])), &Num::Lit(1));

pub static SPEC: CardSpec = CardSpec {
    class: "Dawn",
    // Search your deck for a Basic Pokémon, a Stage 1 Pokémon and a Stage 2 Pokémon, reveal them,
    // put them into your hand, then shuffle.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    predicate: Pred::Pokemon,
                    bounds: Bounds { min: Num::Lit(0), max: Num::Add(&Num::Add(&BASICS, &STAGE1), &STAGE2) },
                    caps: &[Cap { kind: CapKind::Basic, max: BASICS }, Cap { kind: CapKind::Stage1, max: STAGE1 }, Cap { kind: CapKind::Stage2, max: STAGE2 }],
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
