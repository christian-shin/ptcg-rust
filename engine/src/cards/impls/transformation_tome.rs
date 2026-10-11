//! Transformation Tome (CRI 83): you must play 2 Transformation Tome cards at once. (This effect works one time for 2
//! cards.) Choose a Basic Pokémon in your discard pile and switch it with 1 of your Basic Pokémon in play. Any attached
//! cards, damage counters, Special Conditions, turns in play, and any other effects remain on the new Pokémon.
//!
//! "Play 2 at once" is the card's play rule (`CardSpec::together`, user decision D11): both copies are played in one
//! PlayTrainer (one coin under Seismitoad's Quaking Fist: official JP FAQ ガマゲロゲ + 変化の書 「1回投げます」), both go to the play
//! area at once and both are discarded after the effect. The effect (id2372: the new card is the same Pokémon) swaps the chosen discard-pile Basic into the chosen
//! Basic's place at the bottom of its stack (the slot is never empty, so nothing attached is discarded or reset) and the old
//! card goes to the discard pile. A Basic in play counts when it is the top card of its stack (a Fossil played as a Basic
//! counts).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TransformationTome",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        // A Basic Pokémon in play and one in the discard pile (the second copy is the play's companion).
        needs: &[
            Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::Basic),
            Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::Pokemon, Pred::Basic])),
        ],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Basic), msg: "CHOOSE_POKEMON_TO_SWITCH" })),
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::All(&[Pred::Pokemon, Pred::Basic]),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                msg: "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::SwapPokemonCard(SwapPokemonCardSpec { cards: 0, slot: SlotExpr::Picked, into: ZoneRef(Who::Me, Zone::Discard), keep_index: false, bottom: true })),
        ],
    }),
    // "You must play 2 Transformation Tome cards at once."
    together: Some(Pred::Name("Transformation Tome")),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
