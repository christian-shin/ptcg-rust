//! Barbaracle (M3 / POR 43): Stone Arms — once during your turn, attach a
//! Basic [F] Energy from your hand to 1 of your [F] Pokémon. Hammer In — 80.
//!
//! Twinleaf: the Ability throws CANNOT_USE_POWER without a Basic 'Fighting
//! Energy' in the hand (phase 4b R7E: it only checked for a non-empty hand) and
//! the AttachEnergyPrompt takes exactly 1 card (it allowed 0, with the ability
//! used up whatever the answer; ruling 1853); the ability counts as used
//! (marker + board effect) once the prompt is answered.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Barbaracle",
    // Stone Arms: once during your turn, attach a Basic [F] Energy from your hand to 1 of your
    // [F] Pokémon.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("STONE_ARMS_MARKER"),
        needs: &[],
        steps: &[Step::new(Op::Attach(AttachSpec {
            from: ZoneRef(Who::Me, Zone::Hand),
            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]),
            slots: AttachSlots::BenchActive,
            target: Pred::PrintedType(ct::FIGHTING),
            bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
            route: AttachRoute::Move,
            ..AttachSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
