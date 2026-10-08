//! Azumarill ex (MC): Bubble Gathering — as often as you like during your
//! turn, move an Energy from 1 of your other Pokémon to this Pokémon.
//! Energy Balloon — 60+; 40 more for each [P] Energy card attached to this
//! Pokémon.
//!
//! Twinleaf: the MoveEnergyPrompt (cancellable, 0..1) callback throws
//! INVALID_TARGET unless every transfer targets the slot holding this card;
//! ABILITY_USED runs even for an empty answer. Fixed in phase 4b: the prompt
//! now has `blockedFrom` = this Pokémon's slot and `blockedTo` = every other
//! slot (random answers used to reach the INVALID_TARGET throw). Energy
//! Balloon (also fixed: it compared numeric `CardType` enums to 'P', so it
//! never counted anything) runs CheckProvidedEnergyEffect on the Active and
//! adds 40 for every energy map entry that provides [P].
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
            hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::MatchingEntries(ct::PSYCHIC)), &Num::Lit(40)),
            when: Cond::True,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
