//! Ethan's Sudowoodo (DRI 93): Impound — 20; during your opponent's next turn
//! the Defending Pokémon can't retreat. Try to Imitate — flip a coin; if
//! heads, choose 1 of your opponent's Active Pokémon's attacks and use it as
//! this attack.
//!
//! Twinleaf: BLOCK_RETREAT; COIN_FLIP_PROMPT then
//! COPY_OPPONENT_ACTIVE_ATTACK_WITH_RETRY (COPY_ATTACK_FROM_POKEMON_LIST with
//! maxRetries 3, see `copy_attack.rs`). The attack effect is retained across
//! the coin flip.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EthansSudowoodoDRIPool",
    attacks: &[
        // Impound: during your opponent's next turn the Defending Pokémon can't retreat.
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventRetreat }))] },
        // Try to Imitate: flip a coin; if heads, choose 1 of your opponent's Active Pokémon's attacks
        // and use it as this attack.
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::Coin(CoinSpec {
                heads: &[Step::new(Op::CopyAttack(CopyAttackSpec { from: Who::Opp, predicate: Pred::Any, retries: 3 }))],
                ..CoinSpec::DEFAULT
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
