//! Volcanion (POR 25): Strength - 50. Powerful Steam - 90x; flip a coin for each [W] Energy attached to this Pokémon. This attack
//! does 90 damage for each heads.
//!
//! One coin for each unit of [W] Energy this Pokémon provides (as the game checks it), 90 per heads, flipped before the damage
//! (Cottonee PRE 7's counted flips).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "VolcanionPOR",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::before_damage(Op::Coin(CoinSpec {
            flips: Flips::Count(Num::EnergyOn(SlotSel::One(SlotExpr::This), EnergyUnit::Provided(ct::WATER))),
            per_heads: PerHeads::DamageIs(90),
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
