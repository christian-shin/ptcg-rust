//! Stunfisk ex (ASC): Big Bite — 30; during your opponent's next turn the
//! Defending Pokémon can't retreat. Flopping Trap — 100+; 100 more if this
//! Pokémon has any damage counters on it (Twinleaf reads `player.active`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Stunfiskex",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] },
        // Twinleaf reads `player.active` (the Active Pokémon), not this card's own slot.
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(100, Cond::Slot(MY_ACTIVE, SlotPred::Damaged)))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
