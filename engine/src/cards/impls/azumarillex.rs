//! Azumarill ex (MC): Bubble Gathering — as often as you like during your turn, move an Energy from 1 of your other Pokémon
//! to this Pokémon. Energy Balloon — 60+; 40 more for each [P] Energy attached to this Pokémon.
//!
//! Bubble Gathering is `Op::MoveEnergyOwn` with this Pokémon as the only destination (a MoveEnergy between your own
//! Pokémon, APR C-10; cancellable, with no once-per-turn marker). Energy Balloon counts the [P] Energy attached to this
//! Pokémon, which is the Energy provided (`EnergyUnit::Provided`): the text says "Energy", not "Energy cards" (id2237).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Azumarillex",
    // Bubble Gathering: as often as you like during your turn, move an Energy from 1 of your
    // other Pokémon to this Pokémon.
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[Step::new(Op::MoveEnergyOwn(MoveEnergyOwnSpec { to: Some(SlotExpr::This), energy: Pred::Energy, cancel: true, used_always: true }))],
    }],
    // Energy Balloon: 40 more damage for each [P] Energy card attached to this Pokémon.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec {
            op: DamageOp::Add,
            hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::PSYCHIC)), &Num::Lit(40)),
            when: Cond::True,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
