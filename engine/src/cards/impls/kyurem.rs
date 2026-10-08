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
            Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(3), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), max: Num::Min(&Num::Lit(3), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), what: EachWhat::Damage(DamageCalc::Auto), amount: Num::Lit(110), ..EachSlotSpec::DEFAULT })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::AttackCost(AttackCostSpec { change: CostChange::SetCost(&[ct::COLORLESS]), attack: Some(0), subject: SlotPred::Any, guard: Cond::Nonempty(ZoneRef(Who::Opp, Zone::Discard), Pred::All(&[Pred::Trainer, Pred::NameContains("Colress")])), ..AttackCostSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
