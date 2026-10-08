//! Scoop Up Cyclone (TWM, ACE SPEC): put 1 of your Pokémon and all cards
//! attached to it into your hand.
//!
//! Twinleaf: the Trainer play is prevented (the card is never discarded by
//! the card itself); a non-cancellable ChoosePokemonPrompt over the Active and
//! Bench, then one MOVE_POKEMON_OFF_BOARD to the hand.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ScoopUpCyclone",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Me), msg: "CHOOSE_POKEMON_TO_PICK_UP" })),
            Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::Chosen, destination: ZoneRef(Who::Me, Zone::Hand) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
