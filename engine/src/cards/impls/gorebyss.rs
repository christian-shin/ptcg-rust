//! Gorebyss (DRI 56): Crescendo Wave - 30×; this attack does 30 damage for each [W] Energy attached to this Pokémon. Before
//! doing damage, you may attach any number of Basic [W] Energy cards from your hand to this Pokémon.
//!
//! Before the damage: any number (0 to all) of the Basic [W] Energy cards in the hand are chosen and attached to this
//! Pokémon, one Attach event each (Bloodmoon Ursaluna SFA 25's attach); then the damage is 30 times the [W] Energy this
//! Pokémon provides (Darumaka PFL 15, id2237).
use crate::spec::prelude::*;

const WATER_ENERGY: Pred = Pred::All(&[Pred::BasicEnergy, Pred::Name("Water Energy")]);

pub static SPEC: CardSpec = CardSpec {
    class: "Gorebyss",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: WATER_ENERGY,
                bounds: Bounds { min: Num::Lit(0), max: Num::CardCount(ZoneRef(Who::Me, Zone::Hand), WATER_ENERGY) },
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::before_damage(Op::If(IfSpec {
                cond: Cond::Chosen(0),
                yes: &[Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT }))],
                no: &[],
            })),
            Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::WATER)), &Num::Lit(30)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
