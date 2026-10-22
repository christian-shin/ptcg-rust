//! Conkeldurr (BLK 49): Craftsmanship — this Pokémon gets +40 HP for each [F] Energy attached to it. Swing Around — 100+; flip 2
//! coins. This attack does 50 more damage for each heads.
//!
//! Craftsmanship is an Ability `HpMod` over the Holder whose amount counts the [F] Energy this Pokémon provides (Energy attached
//! counts the Energy provided, id2237: a Luminous Energy counts, id2179; a Prism Energy on a Stage 2 provides only [C], id2180).
//! An Ability lock turns it off. Swing Around adds 50 per heads to the damage (APR B-01).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ConkeldurrBLK",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(2), per_heads: PerHeads::DamageAdd(50), ..CoinSpec::DEFAULT }))],
    }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::HpMod(HpModSpec {
            amount: Num::Mul(&Num::EnergyOn(SlotSel::One(SlotExpr::This), EnergyUnit::Provided(ct::FIGHTING)), &Num::Lit(40)),
            subject: SlotPred::Holder,
            guard: Cond::True,
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
