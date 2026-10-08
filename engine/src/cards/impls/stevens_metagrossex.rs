//! Steven's Metagross ex (DRI / ASC): X-Boot - once during your turn, search
//! your deck for a Basic [P] Energy, a Basic [M] Energy, or 1 of each and
//! attach them to your [P] and [M] Pokémon in any way, then shuffle. Metal
//! Stomp - 200.
//!
//! Fixed (phase 4b): AttachEnergyPrompt's validate no longer returns early on
//! validCardTypes, so differentTypes applies here: two Energy of the same
//! type can't be picked (the same-name throw in the callback, which that
//! pair used to reach, is removed).
//!
//! Fixed (phase 4b, R3): the Energy can go only to [P] and [M] Pokémon: the
//! AttachEnergyPrompt's blockedTo lists every other Pokémon (CheckPokemonType
//! on each slot; it used to offer them all).
//!
//! Twinleaf quirks kept: the marker (X_BOOT_MARKER, source this card) and
//! ABILITY_USED are set *before* the prompt; the AttachEnergyPrompt (deck ->
//! Bench + Active, basic Energy, min 0, max 2, differentTypes, validCardTypes
//! [P, M], cancellable) is answered in the callback; each transfer is a
//! MOVE_CARDS (no AttachEnergyEffect), then SHUFFLE_DECK. No deck check, so
//! an empty deck still uses the Ability. The marker is cleared at the end of
//! the turn and on this card's PlayPokemonEffect.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StevensMetagrossex",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("X_BOOT_MARKER"),
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        // Search your deck for a Basic [P] Energy, a Basic [M] Energy, or 1 of each and attach them to your [P] and [M]
        // Pokémon in any way, then shuffle.
        steps: &[
            Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::BenchActive,
                target: Pred::Any,
                scan: TargetScan::EffectiveTypes(&[ct::PSYCHIC, ct::METAL]),
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                same_target: false,
                different_targets: false,
                valid_types: &[ct::PSYCHIC, ct::METAL],
                different_types: true,
                max_per_type: 0,
                cancel: true,
                route: AttachRoute::Move,
                none_shuffles: true,
            })),
            Step::new(Op::If(IfSpec { cond: Cond::Slot(SlotExpr::Picked, SlotPred::Any), yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
