//! Mandibuzz (WHT 64): Look for Prey — once during your turn, your opponent
//! reveals their hand, and you put a Basic Pokémon with 70 HP or less that you
//! find there onto your opponent's Bench. Cutting Wind — 90.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MandibuzzWHTPool",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("WHT_MANDIBUZZ_LOOK_FOR_PREY"),
        // Their open Bench is public knowledge: with it full the Ability can't be used.
        needs: &[Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::Any), Cond::BenchSpace(Who::Opp)],
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::All(&[Pred::Basic, Pred::HpAtMost(70)])),
            yes: &[
                Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::All(&[Pred::Basic, Pred::HpAtMost(70)]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ..PickSpec::DEFAULT })),
                Step::new(Op::PlayFromZone(PlayFromZoneSpec { cards: 0, who: Who::Opp })),
            ],
            no: &[Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), by: Who::Opp, to: Who::Me, when_empty: false }))],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
