//! Tauros (CRI 69): Target Together — choose 1 of your opponent's Pokémon and flip a coin for each of your Pokémon in play that has
//! "Tauros" in its name. This attack does 50 damage to the chosen Pokémon for each heads. (Don't apply Weakness and Resistance for
//! Benched Pokémon.)
//!
//! The Pokémon is chosen first (at step D), then one flip for each of your Pokémon whose card has "Tauros" in its name (the Paldean
//! Tauros and this one count: "in its name" is the card's name, id2255), this Pokémon included; one Damage event of 50 times the heads
//! (none when 0) on the chosen Pokémon, with Weakness and Resistance only if it is the Active Pokémon (APR B-08).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TaurosCRI",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }, purpose: PromptPurpose::DamageTarget })),
            Step::after_damage(Op::Coin(CoinSpec {
                flips: Flips::Count(Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::Top(Pred::NameContains("Tauros")))),
                then: &[Step::new(Op::DamageSlot(DamageSlotSpec {
                    target: SlotTarget::Slot(SlotExpr::Picked),
                    hp: Num::Mul(&Num::Heads, &Num::Lit(50)),
                    target_damage_mul: 0,
                    calc: DamageCalc::Auto,
                    when: Cond::True,
                }))],
                ..CoinSpec::DEFAULT
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
