//! Punk Helmet (PFL, tool): if the [D] Pokémon this card is attached to is in the Active Spot and is damaged by an attack
//! from your opponent's Pokémon (even if this Pokémon is Knocked Out), place 4 damage counters on the Attacking Pokémon.
//!
//! `Event::OnDamagedByAttack` on the Damage event (the Active Spot read when the damage is done, id1992): at step 7 it
//! places 4 counters as a PlaceCounters event on the Attacking Pokémon, caused by the Tool's trigger (counters, not
//! damage: APR C-07), while the Attacking Pokémon is still in play.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PunkHelmet",
    triggers: &[Trigger {
        origin: RuleSource::Tool,
        event: Event::OnDamagedByAttack(OnDamagedByAttackSpec::DEFAULT),
        steps: &[Step::new(Op::If(IfSpec {
            // 4 damage counters on the Attacking Pokémon, if the holder is a [D] Pokémon.
            cond: Cond::Slot(SlotExpr::This, SlotPred::TypeIs(crate::types::ct::DARK)),
            yes: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::Picked), counters: Num::Lit(4) }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
