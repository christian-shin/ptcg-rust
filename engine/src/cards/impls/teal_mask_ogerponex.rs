//! Teal Mask Ogerpon ex (PRE 12 / TWM 25, Tera): Teal Dance - once during your turn, you may attach a Basic [G] Energy card from your
//! hand to this Pokémon. If you attached Energy to a Pokémon in this way, draw a card. Myriad Leaf Shower - 30 + 30 for each Energy
//! attached to both Active Pokémon. Tera: no attack damage while on the Bench.
//!
//! Teal Dance attaches exactly 1 Basic [G] Energy card (the Ability is declined by not using it; JP FAQ: Luminous Energy can't be chosen,
//! usable with an empty deck, the draw can't be declined), and "if you attached" is `Cond::Done` on the Attach (JP FAQ: an attach that
//! didn't happen draws nothing). Myriad Leaf Shower counts the Energy provided (units) on each player's Active Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TealMaskOgerponEx",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        // Attach a Basic [G] Energy card from your hand to this Pokémon; if you did, draw a card.
        steps: &[
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT })),
            // "If you attached Energy in this way, draw a card" (JP FAQ Teal Mask Ogerpon ex: an attach that didn't happen
            // draws nothing; batch 8 C12c).
            Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) }))], no: &[] })),
        ],
    }],
    attacks: &[AttackSpec {
        index: 0,
        // 30 more damage for each Energy attached to both Active Pokémon.
        steps: &[Step::before_damage(Op::Damage(DamageSpec {
            op: DamageOp::Add,
            hp: Num::Mul(&Num::Add(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::EnergyOn(SlotSel::One(OPP_ACTIVE), EnergyUnit::ProvidedUnits)), &Num::Lit(30)),
            when: Cond::True,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
