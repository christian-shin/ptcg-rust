//! Steven's Metagross ex (DRI 145): X-Boot: once during your turn, you may search your deck for a Basic [P] Energy card, a
//! Basic [M] Energy card, or 1 of each and attach them to your [P] Pokémon and [M] Pokémon in any way you like. Then, shuffle
//! your deck. Metal Stomp: 200.
//!
//! An Ability (once per turn). Up to 2 Basic Energy cards of different types ([P] and [M] only) are chosen from the deck and
//! attached, each to a [P] or [M] Pokémon of the player's choice (Attach events, this Ability their cause); the deck is
//! shuffled after the search, whatever was found (APR E-19, user decision D9).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StevensMetagrossex",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        // Search your deck for a Basic [P] Energy, a Basic [M] Energy, or 1 of each and attach them to your [P] and [M]
        // Pokémon in any way, then shuffle.
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::BenchActive,
                target: SlotPred::OneOf(&[SlotPred::TypeIs(ct::PSYCHIC), SlotPred::TypeIs(ct::METAL)]), scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                same_target: false,
                different_targets: false,
                valid_types: &[ct::PSYCHIC, ct::METAL],
                different_types: true,
                max_per_type: 0,
                cancel: true,
                onto: None,
                cards: CardSel::All,
            })),
            // Then shuffle (whatever was found: APR E-19; user decision D9).
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
