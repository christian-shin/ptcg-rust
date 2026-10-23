//! Claydol (CRI 47): Devolution Ray - 50; if your opponent's Active Pokémon is an evolved Pokémon, devolve it by putting the highest
//! Stage Evolution card on it into your opponent's hand.
//!
//! After the damage, one Devolve event by the attack (APR C-13) when the Defending Pokémon is evolved: its top card goes into the
//! opponent's hand (a Pokémon that prevents the effects of attacks stays evolved; the devolved Pokémon keeps its counters and is
//! Knocked Out at the state check if they reach its HP; Espeon ex PRE 34's Amazez).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Claydol",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Slot(OPP_ACTIVE, SlotPred::Evolved),
            yes: &[Step::new(Op::Devolve(DevolveSpec { slot: OPP_ACTIVE, destination: ZoneRef(Who::Opp, Zone::Hand), chooser: None }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
