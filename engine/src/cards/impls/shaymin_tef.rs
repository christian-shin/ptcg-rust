//! Shaymin (TEF 13): Pinpoint Dive — this attack does 60 damage to 1 of your opponent's Benched Pokémon ex or Benched Pokémon V
//! (don't apply Weakness and Resistance for Benched Pokémon). Rear Kick — 50.
//!
//! The attacker chooses among the opponent's Benched Pokémon whose top card is a Pokémon ex or a Pokémon V (step D); the Damage
//! event is caused by the attack and puts the damage as is on the Bench (APR B-08). No such Pokémon: nothing happens.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ShayminTEF",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::DamageSlot(DamageSlotSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Opp), SlotPred::Top(Pred::OneOf(&[Pred::PokemonEx, Pred::PokemonV]))) }),
            hp: Num::Lit(60),
            target_damage_mul: 0,
            calc: DamageCalc::Put,
            when: Cond::True,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
