//! Galvantula (SFA): Compound Eyes — this Pokémon's attacks do 50 more
//! damage to your opponent's Active Pokémon that have an Ability. Shocking
//! Web — 50; 80 more if this Pokémon has any [L] Energy attached.
//!
//! Compound Eyes adds 50 to this Pokémon's attack damage to the opponent's Active Pokémon while that Pokémon has an
//! Ability after effects (`HasAbility`: one whose Abilities an effect removes has none, id141, id2260). Fixed (events
//! batch 6 closeout): the Twinleaf quirks "any power" and "every damage, no target check" are gone. Fixed (phase 4b,
//! R3): Shocking Web sets the damage to 50, +80
//! once if any provided-energy entry is an Energy card printing [L] in
//! `provides` (it used to add 80 per such entry).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Galvantula@SFA",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(more_damage_if(80, Cond::Cmp(Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Me)), EnergyUnit::MatchingEntries(ct::LIGHTNING)), CmpOp::Gt, Num::Lit(0)))),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 50, attacker: SlotPred::IsThisPokemon, opp_active_only: true, guard: Cond::Slot(SlotExpr::Active(Who::Opp), SlotPred::HasAbility), ..DamageDealtSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
