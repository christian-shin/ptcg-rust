//! Emboar (WHT 13): Inferno Fandango - as often as you like during your turn, you may attach a Basic [R] Energy card from your hand
//! to 1 of your Pokémon. Heat Crash - 120.
//!
//! Each use attaches exactly 1 Basic [R] Energy card from the hand (an Attach event caused by the Ability, not the turn's Energy
//! attachment) to the Active or a Benched Pokémon; it can be used again while a card remains (Iono's Bellibolt ex JTG 53's form
//! with any Pokémon as the target). Heat Crash has no text.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Emboar",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[Step::new(Op::Attach(AttachSpec {
            from: ZoneRef(Who::Me, Zone::Hand),
            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Provides(ct::FIRE)]),
            slots: AttachSlots::ActiveBench,
            bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
            ..AttachSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
