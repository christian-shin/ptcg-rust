//! Annihilape (DRI 92): Lose Cool — if this Pokémon has 2 or more damage counters on it, attacks used by this Pokémon do 120 more
//! damage to your opponent's Active Pokémon (before applying Weakness and Resistance). Impact Blow — 160; during your next turn,
//! this Pokémon can't use Impact Blow.
//!
//! Lose Cool is an Ability `DamageDealt` of 120 on the opponent's Active Pokémon, for attacks this Pokémon uses while it has 20
//! or more damage on it, added only to damage above 0 (APR B-04, as Okidogi PRE 57). Impact Blow arms a lasting lock on this
//! Pokémon naming the attack (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AnnihilapeDRI",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))],
    }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 120,
            attacker: SlotPred::IsThisPokemon,
            needs_damage: true,
            guard: Cond::Cmp(Num::DamageOn(SlotExpr::This), CmpOp::Ge, Num::Lit(20)),
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
