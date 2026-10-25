//! Gigalith (WHT 48): Vengeful Cannon - 20x; this attack does 20 damage for each damage counter on all of your Benched [F] Pokémon.
//! Heavy Impact - 160.
//!
//! The damage is 20 for each damage counter on the Benched Pokémon of yours that are [F] as the game checks it (APR D-07), counted
//! before the damage: the damage on them is read in HP (10 per counter), so 20 per counter is twice that sum. With none, the
//! attack does 0 damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Gigalith",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Mul(&Num::DamageSum(SlotSel::Bench(Who::Me), SlotPred::TypeIs(ct::FIGHTING)), &Num::Lit(2))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
