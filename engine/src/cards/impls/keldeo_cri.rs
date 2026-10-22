//! Keldeo (CRI 19): Shoot Through - 20; this attack also does 20 damage to 1 of your opponent's Benched Pokémon. Reflect Energy - 70; move an Energy from this Pokémon to 1 of your Benched Pokémon.
//!
//! Shoot Through's 20 is the attack's damage to a chosen Benched Pokémon (one Damage event, `DamageCalc::Put`; Marnie's Grimmsnarl ex). Reflect Energy moves one Energy from this Pokémon to a Benched Pokémon after the damage (a MoveEnergy event): nothing without a Benched Pokémon or an Energy on it (Iron Treads TEF 118's Wheel Pass).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Keldeo",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp) }),
                hp: Num::Lit(20),
                target_damage_mul: 0,
                calc: DamageCalc::Put,
                when: Cond::True,
            })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
                cond: Cond::All(&[Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any), Cond::AnySlot(SlotSel::One(SlotExpr::This), SlotPred::All(&[SlotPred::IsActive, SlotPred::HasCard(Pred::Energy)]))]),
                yes: &[Step::new(Op::Attach(AttachSpec {
                    from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Active(Who::Me))),
                    predicate: Pred::Energy,
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    ..AttachSpec::DEFAULT
                }))],
                no: &[],
            })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
