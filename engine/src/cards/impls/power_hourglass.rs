//! Powerglass (SFA, tool; class PowerHourglass): at the end of your turn, if
//! the Pokémon this card is attached to is in the Active Spot, you may attach
//! a Basic Energy from your discard pile to that Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PowerHourglass",
    triggers: &[Trigger {
        origin: RuleSource::Tool,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::All(&[Cond::Slot(MY_ACTIVE, SlotPred::Holder), Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), Pred::BasicEnergy)]),
            yes: &[Step::new(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::ActiveOnly,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                ..AttachSpec::DEFAULT
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
