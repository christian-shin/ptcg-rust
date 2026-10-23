//! Fan Rotom (SCR): Fan Call - once during your first turn, you may search your deck for up to 3 [C] Pokémon with 100 HP or
//! less, reveal them, and put them into your hand. Then, shuffle your deck. You can't use more than 1 Fan Call Ability during
//! your turn. Assault Landing - 70; if there is no Stadium in play, this attack does nothing.
//!
//! "Your first turn" is game turn 1 or 2; the shared once-per-turn usage record is what stops a second Fan Call. One Search op
//! revealing what it finds, one shuffle after it (APR E-19). Assault Landing's damage is exactly 0 or 70.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "FanRotom",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(attack_does_nothing(Cond::Not(&Cond::StadiumInPlay(Pred::Any)))),
        ] },
    ],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurnShared,
        needs: &[Cond::Cmp(Num::Turn, CmpOp::Le, Num::Lit(2))],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::COLORLESS), Pred::HpAtMost(100)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, caps: &[Cap { kind: CapKind::Pokemon, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::COLORLESS), Pred::HpAtMost(100)])), &Num::Lit(3)) }], ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
