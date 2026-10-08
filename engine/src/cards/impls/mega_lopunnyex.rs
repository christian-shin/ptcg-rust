//! Mega Lopunny ex (PFL / M2): Gale Thrust — 60+, 170 more if this Pokémon
//! moved from your Bench to the Active Spot this turn. Spiky Hopper — 160,
//! not affected by effects on your opponent's Active Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaLopunnyex",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(170, Cond::ThisMovedToActive))] },
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
