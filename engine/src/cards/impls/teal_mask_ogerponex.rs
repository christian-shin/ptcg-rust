//! Teal Mask Ogerpon ex (TWM, Tera): Teal Dance - once during your turn,
//! attach a Basic [G] Energy card from your hand to this Pokémon; if you
//! did, draw a card. Myriad Leaf Shower - 30 + 30 for each Energy attached
//! to both Active Pokémon. Tera: no attack damage while on the Bench.
//!
//! Fixed (phase 4b, F1): the prompt took 0 cards; a "you may" Ability is declined by not
//! using it, so the attach is exactly 1 Energy (ruling 1778).
//!
//! Twinleaf counts the energy of each player's Active via
//! `CheckProvidedEnergyEffect(player)` (the Active by default).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TealMaskOgerponex",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("TEAL_DANCE_MARKER"),
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::All(&[Pred::BasicEnergy, Pred::Provides(ct::GRASS)]))],
        // Attach a Basic [G] Energy card from your hand to this Pokémon; if you did, draw a card.
        steps: &[
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                msg: "CHOOSE_CARD_TO_ATTACH",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Attached(SlotExpr::This)), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
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
