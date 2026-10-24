//! Excadrill ex (BLK 46): Piercing Drill - 60; this attack also does 60 damage to 1 of your opponent's Benched Pokémon that has any
//! damage counters on it. (Don't apply Weakness and Resistance for Benched Pokémon.) Rock Tumble - 200; this attack's damage isn't
//! affected by Resistance.
//!
//! Piercing Drill: exactly one of the Benched Pokémon that have damage counters is chosen at step D (none when none has: the
//! attack's own damage is still done), 60 damage with no Weakness or Resistance after the damage (Cynthia's Milotic DRI 53's form
//! with a filter). Rock Tumble: the attack ignores Resistance (`AttackFlagKind::NoResistance`, Cynthia's Gible).
use crate::spec::prelude::*;

const DAMAGED_BENCH: SlotSel = SlotSel::Filtered(&SlotSel::Bench(Who::Opp), SlotPred::Damaged);

pub static SPEC: CardSpec = CardSpec {
    class: "ExcadrillEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec {
                among: DAMAGED_BENCH,
                choose: Some(ChooseN {
                    chooser: Who::Me,
                    min: Num::Min(&Num::Lit(1), &Num::SlotCount(DAMAGED_BENCH, SlotPred::Any)),
                    max: Num::Min(&Num::Lit(1), &Num::SlotCount(DAMAGED_BENCH, SlotPred::Any)),
                }),
                what: EachWhat::Damage(DamageCalc::Put),
                amount: Num::Lit(60),
                ..EachSlotSpec::DEFAULT
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
