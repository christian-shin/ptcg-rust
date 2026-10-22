//! Alomomola (BLK 24): Gentle Fin — once during your turn, if this Pokémon is in the Active Spot, you may put a Basic Pokémon with 70 HP
//! or less from your discard pile onto your Bench. Waterfall — 70.
//!
//! The Ability needs this Pokémon in the Active Spot, a Basic Pokémon of 70 HP or less in the discard pile and room on the Bench (an
//! Ability that can't change the game can't be used); the chosen card goes onto the Bench by an EnterPlay event (effect mode, source
//! the discard pile). Once per turn for each copy.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AlomomolaBLK",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::IsActive(SlotExpr::This)],
        steps: &[Step::new(Op::Search(SearchSpec {
            pick: PickSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::All(&[Pred::Pokemon, Pred::Basic, Pred::HpAtMost(70)]),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                ..PickSpec::DEFAULT
            },
            destination: SearchDestination::Bench,
            cancel: false,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
