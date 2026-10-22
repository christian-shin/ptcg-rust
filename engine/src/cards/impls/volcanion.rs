//! Volcanion (POR 25): Strength - 50. Powerful Steam - 90x; flip a coin for each [W] Energy attached to this Pokémon. This attack
//! does 90 damage for each heads.
//!
//! One flip per [W] Energy this Pokémon provides (id2237: Energy attached is the Energy provided; read when the flips start,
//! `Flips::Count(Num)`: Tauros CRI 69), 90 damage for each heads; no [W] Energy: no flip, 0 damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Volcanion",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::before_damage(Op::Coin(CoinSpec {
            flips: Flips::Count(Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::WATER))),
            per_heads: PerHeads::DamageIs(90),
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
