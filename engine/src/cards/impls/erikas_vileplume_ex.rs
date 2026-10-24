//! Erika's Vileplume ex (ASC 3): Lovely Fragrance - once during your turn, you may use this Ability. Heal 30 damage from each of
//! your Pokémon. Bloom Powder - 160; your opponent's Active Pokémon is now Asleep and Poisoned.
//!
//! Lovely Fragrance is once per turn for each copy: a Heal event of 30 on each of your Pokémon, the Active Pokémon first (Mega
//! Floette ex CRI 35's Gentle Light); a Pokémon with no damage is not healed. Bloom Powder: both conditions after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ErikasVileplumeEx",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[Step::new(Op::ForEach(ForEachSpec {
            over: SlotSel::Pokemon(Who::Me),
            body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(30), clear_conditions: false }))],
        }))],
    }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Asleep, SpecialCondition::Poisoned]))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
