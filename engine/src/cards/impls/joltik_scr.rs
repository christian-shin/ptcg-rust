//! Joltik (SCR): Jolting Charge — search your deck for up to 2 Basic [G] Energy cards and up to 2 Basic [L] Energy cards and
//! attach them to your Pokémon in any way you like. Then, shuffle your deck.
//!
//! The Search is one Attach event per Energy from the deck (caused by the attack): 0 to 4 cards, at most 2 of each type, to any
//! of your Pokémon; the search can be declined (a search for a kind). The shuffle follows (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Joltik@SCR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::BenchActive,
                target: Pred::Any,
                scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(4) },
                same_target: false,
                different_targets: false,
                valid_types: &[crate::types::ct::GRASS, crate::types::ct::LIGHTNING],
                max_per_type: 2,
                cancel: true,
                onto: None,
                cards: CardSel::All,
             different_types: false, })),
            Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
