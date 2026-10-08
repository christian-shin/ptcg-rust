//! Dudunsparce ex (JTG): Tenacious Tail — 60× your opponent's Pokémon ex in
//! play. Destructive Drill — 150, not affected by effects on your opponent's
//! Active Pokémon (`ignoreDefenderEffects`; phase 4b R7B: it used to add the
//! damage straight to the Active).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dudunsparceex",
    attacks: &[
        // Tenacious Tail: 60 damage for each of your opponent's Pokémon ex in play.
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(damage_is(Num::Mul(&Num::InPlayCount(Who::Opp, PlayScope::All, Pred::Tag(crate::types::tag::POKEMON_EX_LOWER)), &Num::Lit(60))))],
        },
        // Destructive Drill: not affected by any effects on your opponent's Active Pokémon.
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
