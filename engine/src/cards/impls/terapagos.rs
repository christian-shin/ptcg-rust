//! Terapagos (SSP 161): Prism Charge - search your deck for up to 3 Basic Energy cards of different types and attach them to your
//! Tera Pokémon in any way you like. Then, shuffle your deck. Hard Tackle - 100.
//!
//! One Attach of 0 to 3 Basic Energy cards of different types from the deck onto your Pokémon carrying the Tera tag (an Attach
//! event per card, caused by the attack), then one shuffle (APR E-19; Crispin SCR 133's different types).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Terapagos",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Attach(AttachSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::BasicEnergy,
                    slots: AttachSlots::ActiveBench,
                    target: SlotPred::Tag(crate::types::tag::POKEMON_TERA),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                    different_types: true,
                    ..AttachSpec::DEFAULT
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
