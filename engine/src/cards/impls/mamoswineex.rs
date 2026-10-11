//! Mamoswine ex (JTG): Mammoth Hauler - once during your turn, you may search your deck for a Pokémon, reveal it, and put it into
//! your hand. Then, shuffle your deck. Rumbling March - 180+; 40 more damage for each Stage 2 Pokémon on your Bench.
//!
//! One Search op revealing what it finds, then one shuffle after it (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Mamoswineex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Add(
            &Num::Lit(180),
            &Num::Mul(&Num::SlotCount(SlotSel::Bench(Who::Me), SlotPred::Top(Pred::StageIs(crate::types::Stage::Stage2))), &Num::Lit(40)),
        )))],
    }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("MAMMOTH_RIDE_MARKER"),
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
