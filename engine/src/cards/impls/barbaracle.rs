//! Barbaracle (M3 / POR 43): Stone Arms — once during your turn, attach a Basic [F] Energy from your hand to 1 of your [F]
//! Pokémon. Hammer In — 80.
//!
//! Stone Arms is an Ability used once per turn: exactly 1 Basic Fighting Energy card from the hand, attached (an Attach
//! event with the Ability as its cause) to a Pokémon that is [F] as the game checks its type (`TargetScan::EffectiveTypes`).
//! It can't be used without a Basic [F] Energy card in the hand (id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Barbaracle",
    // Stone Arms: once during your turn, attach a Basic [F] Energy from your hand to 1 of your
    // [F] Pokémon.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[Step::new(Op::Attach(AttachSpec {
            target: SlotPred::TypeIs(ct::FIGHTING),
            from: ZoneRef(Who::Me, Zone::Hand),
            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]),
            slots: AttachSlots::BenchActive,
            bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
            ..AttachSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
