//! Transformation Tome / Book of Transformation (CRI): play 2 at once.
//! Choose a Basic Pokémon in your discard pile and switch it with 1 of your
//! Basic Pokémon in play (attachments, damage and effects stay).
//!
//! Twinleaf: "Basic in play" is a slot whose top card is Basic (phase 4b
//! fix: it used to count a Basic under an evolution, so with only evolved
//! Pokémon in play the card was playable but every slot was blocked and the
//! prompt unanswerable); a Fossil in play (a Trainer card played as a Basic
//! Pokémon) counts, and the in-play choice blocks slots whose top card isn't
//! Basic. Phase 4b fix (rulings 1840, card text "attached cards, damage counters,
//! Special Conditions, turns in play, and any other effects remain on the new
//! Pokémon"): the discard Basic goes onto the slot first and the old bottom card
//! is discarded after (the slot is never empty, so nothing is discarded or reset;
//! Twinleaf used to empty the slot first), the new card takes the old card's place
//! at the bottom of the stack and gets its `damageTakenLastTurn` and
//! `movedToActiveThisTurn` (card flag and the player's id lists). The second copy
//! is discarded from hand.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TransformationTome",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        // Play 2 at once: a second copy in the hand, a Basic Pokémon in play and one in the discard pile.
        needs: &[
            Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Name("Transformation Tome")),
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
            // The second copy is discarded from your hand.
            Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::First(Pred::Name("Transformation Tome")), ..DiscardSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
