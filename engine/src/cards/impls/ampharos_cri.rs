//! Ampharos (CRI 29): Synchro Pulse — if you have the same number of cards in your hand as your opponent, attacks used by this Pokémon
//! do 80 more damage to your opponent's Active Pokémon (before applying Weakness and Resistance). Flashing Bolt — 140; during your next
//! turn, this Pokémon can't use Flashing Bolt.
//!
//! Synchro Pulse is an Ability `DamageDealt` of 80 on the opponent's Active Pokémon, for attacks this Pokémon uses while both hands
//! have the same number of cards, added only to damage above 0 (APR B-04, as Annihilape DRI 92). Flashing Bolt arms a lasting lock
//! on this Pokémon naming the attack (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AmpharosCRI",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))],
    }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 80,
            attacker: SlotPred::IsThisPokemon,
            needs_damage: true,
            guard: Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Eq, Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand))),
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
