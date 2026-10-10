//! Munkidori (TWM): Adrena-Brain — once during your turn, if this Pokémon has
//! any [D] Energy attached, move up to 3 damage counters from 1 of your
//! Pokémon to 1 of your opponent's Pokémon. Mind Bend — Confused.
//!
//! Adrena-Brain is one MoveCounters event (one use is one action, id67; both ends are asked): up to 3 counters leave the
//! source Pokémon and are placed on the destination, as many as the source has (id63). Patrat's Watchful Eye stops it
//! (id2350); a protection on the destination that reads an Ability's cause (Hide 'n' Sneak, Battle Cage on a Benched
//! Pokémon) makes the counters vanish, Mist Energy doesn't (it protects against attacks only). It is neither healing nor
//! damage (id1995). Mind Bend's Confusion is a GainCondition with the attack as its cause.
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
