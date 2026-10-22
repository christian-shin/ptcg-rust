//! Mega Zygarde ex (POR 47): Gaia Wave — 200; during your opponent's next turn, this Pokémon takes 30 less damage from attacks (after
//! applying Weakness and Resistance). Nullifying Zero — for each of your opponent's Pokémon, flip a coin. If heads, this attack does
//! 150 damage to that Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! Gaia Wave arms the lasting "takes less damage". Nullifying Zero flips one coin for each and every Pokémon the opponent has in play,
//! in turn, and each heads is one Damage event of 150 on that Pokémon (Weakness and Resistance only for the Active Pokémon, APR B-08);
//! the order doesn't matter (id2316; a Tera Pokémon on the Bench takes none, the Tera rule).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaZygardeEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(30, DamageSource::Any) }))] },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::ForEach(ForEachSpec {
                over: SlotSel::Pokemon(Who::Opp),
                body: &[Step::new(Op::Coin(CoinSpec {
                    heads: &[Step::new(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(150), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True }))],
                    ..CoinSpec::DEFAULT
                }))],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
