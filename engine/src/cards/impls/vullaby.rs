//! Vullaby (RCL): Pluck — 10; before doing damage, discard all Pokémon Tools
//! from your opponent's Active Pokémon (one MOVE_CARDS per Tool).
//!
//! Spec: a plain move per Tool (today's behavior; making it an attack effect that Mist Energy stops is planned change B-PC-13).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Vullaby",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Tools(SlotExpr::Active(Who::Opp))), ..DiscardSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
