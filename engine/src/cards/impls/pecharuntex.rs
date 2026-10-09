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
            Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::All(&[SlotPred::PrintedTypeIs(crate::types::ct::DARK), SlotPred::Not(&SlotPred::Named("Pecharunt ex"))])),
            Cond::Cmp(Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::PrintedTypeIs(crate::types::ct::DARK)), CmpOp::Gt, Num::Lit(1)),
        ],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec {
                chooser: Who::Me,
                among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::All(&[SlotPred::PrintedTypeIs(crate::types::ct::DARK), SlotPred::Not(&SlotPred::Named("Pecharunt ex"))])),
                msg: "CHOOSE_POKEMON_TO_SWITCH",
            })),
            Step::new(Op::Switch(SwitchSpec { side: Who::Me, chooser: Who::Me, kind: SwitchKind::Picked, msg: "", required: false })),
            Step::new(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Poisoned]), gate: Gate::None, when: Cond::True })),
            Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "CHAINS_OF_CONTROL_USED_MARKER", source: RuleSource::Ability })),
        ],
    }],
    triggers: &[Trigger {
        origin: RuleSource::CardRule,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "CHAINS_OF_CONTROL_USED_MARKER", from: MarkerFrom::Any }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
