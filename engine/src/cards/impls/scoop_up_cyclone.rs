//! Scoop Up Cyclone (TWM, ACE SPEC): put 1 of your Pokémon and all cards
//! attached to it into your hand.
//!
//! A non-cancellable choice among your Active and Benched Pokémon, then `Op::RemoveFromPlay` to the hand: one LeavePlay
//! event of that Pokémon (the whole stack) with the Item as its cause.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ScoopUpCyclone",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Me), msg: "CHOOSE_POKEMON_TO_PICK_UP" })),
            Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::Picked, destination: ZoneRef(Who::Me, Zone::Hand) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
