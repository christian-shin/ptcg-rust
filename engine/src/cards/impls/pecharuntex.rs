//! Pecharunt ex (SFA): Subjugating Chains - once during your turn, switch 1
//! of your Benched [D] Pokémon (except Pecharunt ex) with your Active; the
//! new Active is Poisoned. Irritated Outburst - 60x prizes the opponent took.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Pecharuntex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(damage_is(Num::Mul(&Num::PrizesTaken(Who::Opp), &Num::Lit(60))))] }],
    powers: &[PowerSpec {
        index: 0,
        // One use per turn for the player, whichever copy used it.
        once: Once::No,
        needs: &[
            Cond::Not(&Cond::Used(UsedSpec::ability(0, UsedBy::Player))),
            // A Benched [D] Pokémon that isn't a Pecharunt ex to switch in (the current type is a checked read, which the
            // usability walk of the pick below only assumes possible).
            Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::All(&[SlotPred::TypeIs(crate::types::ct::DARK), SlotPred::Not(&SlotPred::Named("Pecharunt ex"))])),
        ],
        steps: &[
            Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::All(&[SlotPred::TypeIs(crate::types::ct::DARK), SlotPred::Not(&SlotPred::Named("Pecharunt ex"))])) }, purpose: PromptPurpose::SwitchPokemon })),
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Picked, required: false })),
            // If you do, the new Active Pokémon is now Poisoned.
            Step::new(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Poisoned]), when: Cond::Done })),
            Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Player))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
