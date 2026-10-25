//! Hydreigon ex (WHT 67): Greedy Eater - if your opponent's Basic Pokémon is Knocked Out by damage from an attack used by this Pokémon,
//! take 1 more Prize card. Dark Bite - 200; during your opponent's next turn, the Defending Pokémon can't retreat. This is a different
//! card from Hydreigon ex SSP 119.
//!
//! Greedy Eater is a `PrizeAdjust` of +1 over a Knock Out by attack damage of a Basic Pokémon on the other side whose attacker is this
//! Pokémon (where it is now). Dark Bite arms the lasting `CANT_RETREAT` lock on the Defending Pokémon (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HydreigonExWHT",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec {
            delta: 1,
            subject: SlotPred::All(&[SlotPred::Basic, SlotPred::Not(&SlotPred::OnMySide)]),
            by_attack_damage: true,
            attacker: SlotPred::IsThisPokemon,
            ..PrizeAdjustSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
