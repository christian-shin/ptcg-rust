//! Ethan's Sudowoodo (DRI 93): Impound — 20; during your opponent's next turn the Defending Pokémon can't retreat. Try to Imitate —
//! flip a coin; if heads, choose 1 of your opponent's Active Pokémon's attacks and use it as this attack.
//!
//! Impound arms a lasting `CANT_RETREAT` lock on the Defending Pokémon. Try to Imitate flips the coin before the damage; on
//! heads `Op::CopyAttack` over the opponent's Active Pokémon's attacks, chosen by the player once; the copied attack is this
//! Pokémon's only while it is used, and an attack that can't be fully done is still used and ends (RULES.md "A copied attack is
//! the copycat's only while it is used").
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EthansSudowoodo",
    attacks: &[
        // Impound: during your opponent's next turn the Defending Pokémon can't retreat.
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] },
        // Try to Imitate: flip a coin; if heads, choose 1 of your opponent's Active Pokémon's attacks
        // and use it as this attack.
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::Coin(CoinSpec {
                heads: &[Step::new(Op::CopyAttack(CopyAttackSpec { from: Who::Opp, predicate: Pred::Any, scope: CopyScope::Active }))],
                ..CoinSpec::DEFAULT
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
