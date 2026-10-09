//! Munkidori (TWM): Adrena-Brain — once during your turn, if this Pokémon has
//! any [D] Energy attached, move up to 3 damage counters from 1 of your
//! Pokémon to 1 of your opponent's Pokémon. Mind Bend — Confused.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Munkidori",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Confused]))] }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("ADRENA_BRAIN_MARKER"),
        needs: &[Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::Damaged), Cond::Slot(SlotExpr::This, SlotPred::Provides(crate::types::ct::DARK))],
        steps: &[Step::new(Op::MoveCounters(MoveCountersSpec { kind: MoveCountersKind::MineToOpp { max: 3 } }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
