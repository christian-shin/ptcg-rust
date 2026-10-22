//! Delibird (CRI 18): Pleasing Present - each player may attach up to 3 Basic Energy cards from their hand to their Pokémon in
//! any way they like. Your opponent does this first. Flap - 40.
//!
//! Two Attach ops, the opponent's first: each player chooses up to 3 Basic Energy cards of their own hand (0 allowed) and the
//! Pokémon of their own to attach them to, an Attach event per card caused by the attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DelibirdCRI",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Attach(AttachSpec {
                chooser: Who::Opp,
                from: ZoneRef(Who::Opp, Zone::Hand),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::ActiveBench,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                ..AttachSpec::DEFAULT
            })),
            Step::after_damage(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Hand),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::ActiveBench,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                ..AttachSpec::DEFAULT
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
