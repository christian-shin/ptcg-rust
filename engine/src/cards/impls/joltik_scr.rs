//! Joltik (SCR): Jolting Charge — search your deck for up to 2 Basic [G]
//! Energy and up to 2 Basic [L] Energy and attach them to your Pokémon in
//! any way you like, then shuffle.
//!
//! Twinleaf: a cancellable AttachEnergyPrompt on the deck (basic Energy,
//! max 4, validCardTypes [G, L], maxPerType 2; the old differentTypes option
//! is dropped: it limited the answer to one Energy per type once the validCardTypes
//! early return was fixed, but the card allows 2 of each). No transfer
//! → SHUFFLE_DECK; otherwise a MOVE_CARDS per transfer, then a
//! ShuffleDeckPrompt.
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
                route: AttachRoute::Move,
                onto: None,
                cards: CardSel::All,
                none_shuffles: false,
             different_types: false, })),
            Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
