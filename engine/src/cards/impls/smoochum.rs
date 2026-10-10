//! Smoochum (SSP): Delightful Kiss ("Happy Kiss") — search your deck for up to
//! 2 Basic [P] Energy cards and attach them to 1 of your Benched Pokémon.
//! Then, shuffle your deck.
//!
//! Fixed (phase 4b, W4): the prompt let the two Energy go to different
//! Benched Pokémon; it now requires the same target (`sameTarget`).
//!
//! An empty deck makes the attack do nothing (it is still usable: rulings 337 and 1790). The printed order: the search
//! (an Attach from the deck, 0..2, no cancel), then one shuffle (APR E-19; user decision D9: Twinleaf opened its shuffle
//! before the attach answer and shuffled again when nothing was attached).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Smoochum",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::If(IfSpec { cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), yes: &[Step::new(Op::Attach(AttachSpec {
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
             different_types: false, })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
