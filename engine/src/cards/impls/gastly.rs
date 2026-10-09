//! Gastly (EVO): Little Grudge — during your opponent's next turn, if this
//! Pokémon is Knocked Out by damage from an attack, discard an Energy
//! attached to the Attacking Pokémon. Nightmare — 20; flip a coin, if heads
//! your opponent's Active Pokémon is now Asleep.
//!
//! Twinleaf: Little Grudge arms the slot fields through a
//! DiscardAttackerEnergyIfKnockedOut EffectOfAttack (resolved in the
//! KnockOutEffect reducer); Nightmare flips after the damage and the
//! Special Condition is an effect of the attack (Mist Energy prevents it).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Gastly@EVO",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::DiscardAttackerEnergyIfKnockedOut })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Coin(CoinSpec { before: Cond::True, heads: &[Step::new(inflict(&[SpecialCondition::Asleep]))], ..CoinSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
