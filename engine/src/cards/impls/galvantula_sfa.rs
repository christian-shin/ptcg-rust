//! Galvantula (SFA 2): Compound Eyes — attacks used by this Pokémon do 50 more damage to your opponent's Active Pokémon that has
//! an Ability (before applying Weakness and Resistance). Shocking Web — 50+; if this Pokémon has any [L] Energy attached, 80
//! more damage.
//!
//! Compound Eyes is a `Modifier::DamageDealt` at the attack stage (attacker `IsThisPokemon`, the opponent's Active only) whose
//! guard reads the Defending Pokémon's Abilities after effects (`SlotPred::HasAbility`: a Pokémon whose Abilities an effect
//! removes has none, id141, id2260). Shocking Web adds 80 once when this Pokémon has any [L] Energy provided
//! (`EnergyUnit::Provided`: "Energy attached" counts the Energy provided, RULES.md).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Galvantula",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(more_damage_if(80, Cond::Cmp(Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Me)), EnergyUnit::Provided(ct::LIGHTNING)), CmpOp::Gt, Num::Lit(0)))),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 50, attacker: SlotPred::IsThisPokemon, opp_active_only: true, guard: Cond::Slot(SlotExpr::Active(Who::Opp), SlotPred::HasAbility), ..DamageDealtSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
