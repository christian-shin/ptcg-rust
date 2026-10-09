//! Poltchageist (PBL / M5): Hide 'n' Sneak. Furtive Drop — place 1 damage
//! counter on your opponent's Active Pokémon (a PutCountersEffect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Poltchageist",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(OPP_ACTIVE), counters: Num::Lit(1) }))],
    }],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(HIDE_N_SNEAK) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
