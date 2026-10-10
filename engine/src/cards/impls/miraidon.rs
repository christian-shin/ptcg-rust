//! Miraidon (TEF): Peak Acceleration — 40; search your deck for up to 2 Basic
//! Energy and attach them to your Future Pokémon in any way, then shuffle.
//! Sparking Strike — 160.
//!
//! Twinleaf: AttachEnergyPrompt over the deck (Bench + Active, no cancel,
//! min 0, max 2). blockedTo lists every non-Future Pokémon (phase 4b fix: it
//! used to offer them and the Energy then threw INVALID_TARGET). No transfers
//! -> SHUFFLE_DECK. Otherwise each transfer is checked in order
//! (`target.cards[0]` must be a Future Pokémon, else throws INVALID_TARGET
//! after the earlier ones already moved) and the Energy is moved with
//! MOVE_CARDS (no AttachEnergyEffect); then a bare ShuffleDeckPrompt (no
//! trailing wait prompt).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Miraidon",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::BenchActive,
                target: Pred::Tag(crate::types::tag::FUTURE),
                scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                route: AttachRoute::Move,
                onto: None,
                cards: CardSel::All,
                none_shuffles: false,
             different_types: false, })),
            Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
