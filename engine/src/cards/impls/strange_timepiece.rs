//! Strange Timepiece (MEG): devolve 1 of your evolved [P] Pokémon by putting
//! any number of Evolution cards on it into your hand.
//!
//! Twinleaf: the second prompt is a ChooseCardsPrompt over the whole slot
//! (Pokémon filter) with the Basic's index blocked (phase 4b fix: the Basic
//! used to be selectable and threw INVALID_PROMPT_RESULT; the throw stays as
//! a defensive check); choosing a card devolves `pokemons.length - index` times via
//! DEVOLVE_POKEMON (which sets `pokemonPlayedTurn` but not
//! `cannotEvolveNextTurn`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StrangeTimepiece",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        // One of your evolved [P] Pokémon.
        needs: &[Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Evolved, SlotPred::TypeIs(ct::PSYCHIC)]))],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec {
                chooser: Who::Me,
                among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Evolved, SlotPred::TypeIs(ct::PSYCHIC)])),
                msg: "CHOOSE_POKEMON",
            })),
            Step::new(Op::Devolve(DevolveSpec { slot: SlotExpr::Picked, destination: ZoneRef(Who::Me, Zone::Hand), chooser: Some(Who::Me) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
