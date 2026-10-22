//! Landorus (BLK 53): Abundant Harvest - attach a Basic [F] Energy card from your discard pile to this Pokémon. Earthquake -
//! 110; this attack also does 10 damage to each of your Benched Pokémon. (Don't apply Weakness and Resistance for Benched
//! Pokémon.)
//!
//! Abundant Harvest: one Attach onto this Pokémon of a Basic [F] Energy from the discard pile (nothing when none).
//! Earthquake: one Damage event per Benched Pokémon of yours, no Weakness or Resistance (APR B-08; Centiskorch SSP 28).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LandorusBLK",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Discard),
                    predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                    into: 0,
                    ..PickSpec::DEFAULT
                })),
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Chosen(0),
                    yes: &[Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT }))],
                    no: &[],
                })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Bench(Who::Me), what: EachWhat::Damage(DamageCalc::Put), amount: Num::Lit(10), when: Cond::True, ..EachSlotSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
