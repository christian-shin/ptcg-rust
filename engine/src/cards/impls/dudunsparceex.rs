//! Dudunsparce ex (JTG 121): Tenacious Tail — 60× your opponent's Pokémon ex in play. Destructive Drill — 150; this attack's
//! damage isn't affected by any effects on your opponent's Active Pokémon.
//!
//! Tenacious Tail sets the damage to 60 per Pokémon ex of the opponent's in play. Destructive Drill raises the attack flag
//! `AttackFlagKind::IgnoreDefenderEffects` before the damage: the damage calculation leaves out every effect on the
//! Defending Pokémon (id1877: a "+90 more damage" effect on it isn't added).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DudunsparceEx",
    attacks: &[
        // Tenacious Tail: 60 damage for each of your opponent's Pokémon ex in play.
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(damage_is(Num::Mul(&Num::InPlayCount(Who::Opp, PlayScope::All, Pred::PokemonEx), &Num::Lit(60))))],
        },
        // Destructive Drill: not affected by any effects on your opponent's Active Pokémon.
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
