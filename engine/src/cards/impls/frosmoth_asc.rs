//! Frosmoth (ASC 53): Alluring Wings - once during your turn, if this Pokémon is in the Active Spot, you may use this
//! Ability. Each player draws a card. Cold Cyclone - 90; move a [W] Energy from this Pokémon to 1 of your Benched Pokémon.
//!
//! Alluring Wings is one Draw event for each player (cause: this Ability; Mega Kangaskhan ex MEG 104's Run Errand with the
//! opponent drawing too), usable in the Active Spot when a card can be drawn. Cold Cyclone moves one [W] Energy card from
//! this Pokémon to a Benched Pokémon after the damage (a MoveEnergy event; Iron Treads TEF 118's Wheel Pass with a type):
//! nothing without a Benched Pokémon or a [W] Energy card.
use crate::spec::prelude::*;

const WATER_ENERGY_CARD: Pred = Pred::All(&[Pred::Energy, Pred::Provides(ct::WATER)]);

pub static SPEC: CardSpec = CardSpec {
    class: "FrosmothASC",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::IsActive(SlotExpr::This)],
        steps: &[
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
            Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::All(&[Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any), Cond::AnySlot(SlotSel::One(SlotExpr::This), SlotPred::All(&[SlotPred::IsActive, SlotPred::HasCard(WATER_ENERGY_CARD)]))]),
            yes: &[Step::new(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Active(Who::Me))),
                predicate: WATER_ENERGY_CARD,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                ..AttachSpec::DEFAULT
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
