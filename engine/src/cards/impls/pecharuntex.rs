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
            Cond::Not(&Cond::HasMarker { who: Who::Me, name: "CHAINS_OF_CONTROL_USED_MARKER", from: MarkerFrom::Any }),
        ],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec {
                chooser: Who::Me,
                among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::All(&[SlotPred::PrintedTypeIs(crate::types::ct::DARK), SlotPred::Not(&SlotPred::Named("Pecharunt ex"))])),
                msg: "CHOOSE_POKEMON_TO_SWITCH",
            })),
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Picked, msg: "", required: false })),
            // If you do, the new Active Pokémon is now Poisoned.
            Step::new(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Poisoned]), when: Cond::Done })),
            Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "CHAINS_OF_CONTROL_USED_MARKER", source: RuleSource::Ability })),
        ],
    }],
    triggers: &[Trigger {
        origin: RuleSource::CardRule,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EndTurn), EventPred::Owner(Who::Me)])),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "CHAINS_OF_CONTROL_USED_MARKER", from: MarkerFrom::Any }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
