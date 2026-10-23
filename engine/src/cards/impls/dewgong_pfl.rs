//! Dewgong (PFL 22): Thick Fat - this Pokémon takes 30 less damage from attacks from your opponent's [R] or [W] Pokémon (after
//! applying Weakness and Resistance). Slam - 70x; flip 2 coins. This attack does 70 damage for each heads.
//!
//! Thick Fat is an Ability `DamageTaken` of 30 on this Pokémon, from the opponent's attacks whose attacking Pokémon is [R] or [W]
//! as the game checks its type (`SlotPred::TypeIs`, APR D-07; Mudsdale DRI 108's Mud Coat with an attacker type). Slam flips 2 coins
//! before the damage, 70 per heads.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DewgongPFL",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(Num::Lit(2)), per_heads: PerHeads::DamageIs(70), ..CoinSpec::DEFAULT }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageTaken(DamageTakenSpec {
            amount: 30,
            subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
            source: SlotPred::OneOf(&[SlotPred::TypeIs(ct::FIRE), SlotPred::TypeIs(ct::WATER)]),
            ..DamageTakenSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
