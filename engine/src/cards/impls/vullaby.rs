//! Vullaby (RCL): Pluck - 10; before doing damage, discard all Pokémon Tools from your opponent's Active Pokémon.
//!
//! The Tools leave play to the discard pile before the damage: a LeavePlay of each Tool (D1), caused by this attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Vullaby",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Tools(SlotExpr::Active(Who::Opp))), ..DiscardSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
