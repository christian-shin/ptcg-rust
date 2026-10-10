//! Smoochum (SSP): Delightful Kiss ("Happy Kiss") — search your deck for up to
//! 2 Basic [P] Energy cards and attach them to 1 of your Benched Pokémon.
//! Then, shuffle your deck.
//!
//! Fixed (phase 4b, W4): the prompt let the two Energy go to different
//! Benched Pokémon; it now requires the same target (`sameTarget`).
//!
//! Twinleaf: an empty deck makes the attack do nothing (it is still usable:
//! phase 4b R7E, rulings 337 and 1790; it used to throw CANNOT_USE_ATTACK).
//! Opens an AttachEnergyPrompt (0..2, no cancel) and, without waiting for it, a
//! ShuffleDeckPrompt whose callback applies the order (no trailing wait). The
//! attach callback shuffles again (SHUFFLE_DECK) when nothing was attached.
//!
//! Spec: the shuffle comes first (Twinleaf opens its shuffle before the attach answer, which the replay's shuffle tape matches by deck size).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Smoochum",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::If(IfSpec { cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })), Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Psychic Energy")]),
                slots: AttachSlots::Bench,
                target: Pred::Any,
                scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                same_target: true,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                onto: None,
                cards: CardSel::All,
                none_shuffles: true,
             different_types: false, }))], no: &[] }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
