//! Aurorus (POR 24): Tundra Wall - all of your Pokémon that have any [W] Energy attached take 50 less damage from attacks from your
//! opponent's Pokémon (after applying Weakness and Resistance). The effect of Tundra Wall doesn't stack. Freezing Chill - 150;
//! during your opponent's next turn, the Defending Pokémon can't use attacks.
//!
//! Tundra Wall is an Ability `DamageTaken` over the owner's Pokémon that provide [W] (the Energy attached, as the game checks it),
//! never reducing the owner's own attacks' damage, non-stacking per Ability name (`NonStack::SameAbility`; Steven's Carbink DRI 86).
//! Freezing Chill arms the lasting `CANT_ATTACK` lock on the Defending Pokémon (Beartic BLK 26).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Aurorus",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_ATTACK)) }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 50, subject: SlotPred::Provides(ct::WATER), side: Side::Owner, nonstacking: Some(NonStack::SameAbility), ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
