//! Kyurem (SFA): Plasma Bane — if the opponent has a card with "Colress" in
//! its name in their discard pile, Trifrost costs only [C]. Trifrost —
//! discard all Energy from this Pokémon; 110 damage to 3 of the opponent's
//! Pokémon.
//!
//! Fixed (phase 4b, W4): the target prompt was min 1 max 3; it is now exactly
//! min(3, opponent's Pokémon in play).
//!
//! Fixed (phase 4b, R7F-11; ruling 1581): the cost [C] was an ordinary cost
//! that Pokémon League Headquarters, Rillaboom's Drum Beating, Antique Root
//! Fossil, Counter Gain, ... still changed; the Ability now sets it
//! (CheckAttackCostEffect.setCost), and a set cost is not increased or
//! decreased.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Kyurem",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::DamageChosen(DamageChosenSpec { among: SlotSel::Pokemon(Who::Opp), count: 3, hp: Num::Lit(110), calc: DamageCalc::Auto, msg: "CHOOSE_POKEMON_TO_DAMAGE" })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotExpr::Active(Who::Me), selection: EnergySelection::AllProvided })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::AttackCost(AttackCostSpec { change: CostChange::SetCost(&[ct::COLORLESS]), attack: Some(0), subject: SlotPred::Any, guard: Cond::Nonempty(ZoneRef(Who::Opp, Zone::Discard), Pred::All(&[Pred::Trainer, Pred::NameContains("Colress")])), ..AttackCostSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
