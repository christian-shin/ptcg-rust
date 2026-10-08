//! Zapdos (TWM): Thunder Wave — flip a coin, if heads the opponent's Active
//! Pokémon is now Paralyzed. Thunderbolt — 190; discard all Energy from this
//! Pokémon.
//!
//! Twinleaf: the coin callback reduces an AddSpecialConditionsEffect; the
//! discard is one DiscardCardsEffect with every card of the Active's
//! CheckProvidedEnergy map, aimed at `player.active`.
//!
//! Fixed (phase 4b, R2): Thunderbolt discarded the Energy in the attack
//! handler, before the damage (Voltaic Lightning Energy's +20 was lost); it
//! now discards in AfterAttackEffect with a fresh AttackEffect's data.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zapdos",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed], Cause::Attack))], ..CoinSpec::DEFAULT }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: MY_ACTIVE, selection: EnergySelection::AllProvided }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
