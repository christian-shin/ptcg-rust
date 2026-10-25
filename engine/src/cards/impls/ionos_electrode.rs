//! Iono's Electrode (JTG 48): Thump-Thump Boom - this Pokémon does 100 damage to itself. Flip a coin. If heads, your opponent's Active
//! Pokémon is Knocked Out. Electric Ball - 100.
//!
//! The self-damage is a full calculation on this Pokémon (its Weakness and Resistance apply, APR B-09) after the attack's damage; the
//! coin follows, and on heads the Defending Pokémon is Knocked Out by the attack's effect (one KnockOut event, at the next state check
//! with this Pokémon's own, Alolan Exeggutor ex SSP 133's Swinging Sphene).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "IonosElectrode",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(self_damage(100)),
            Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(Op::KnockOut(KnockOutSpec { target: OPP_ACTIVE, when: Cond::True }))], ..CoinSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
