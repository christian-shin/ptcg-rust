//! Paldean Tauros (PFL 48): Raging Charge — 40×; this attack does 40 damage for each of your Pokémon that has "Tauros" in its
//! name that has any damage counters on it. Double-Edge — 70; this Pokémon also does 20 damage to itself.
//!
//! Raging Charge counts every damaged Pokémon of yours whose name contains "Tauros", Paldean Tauros included (id2255, JP FAQ
//! Paldean Tauros SSP 39). Double-Edge's self-damage is a Damage event on this Pokémon caused by the attack (APR B-09).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PaldeanTaurosPFL48",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(damage_is(Num::Mul(
                &Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Top(Pred::NameContains("Tauros")), SlotPred::Damaged])),
                &Num::Lit(40),
            )))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(self_damage(20))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
