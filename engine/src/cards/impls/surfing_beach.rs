//! Surfing Beach (M1S / CRI, stadium): once during each player's turn, that
//! player may switch their Active [W] Pokémon with 1 of their Benched [W]
//! Pokémon.
//!
//! Twinleaf quirks kept: the `blocked` list gets the Active and every Bench
//! slot that is not [W] (the `else` binds to the second `if`); the card
//! throws CANNOT_USE_STADIUM without an Active and a Benched [W] Pokémon or
//! when stadium effects on the Active are blocked; the callback re-checks the
//! chosen slot, then switches silently (`player.switchPokemon(target)`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SurfingBeach",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        // An Active [W] Pokémon and a Benched [W] Pokémon, and the Stadium's effect not blocked on the Active.
        needs: &[
            Cond::Slot(MY_ACTIVE, SlotPred::TypeIs(ct::WATER)),
            Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::TypeIs(ct::WATER)),
            Cond::Not(&Cond::StadiumBlocked(MY_ACTIVE)),
        ],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::TypeIs(ct::WATER)), msg: "CHOOSE_NEW_ACTIVE_POKEMON" })),
            Step::new(Op::Switch(SwitchSpec { side: Who::Me, chooser: Who::Me, kind: SwitchKind::ChosenSilent, msg: "", required: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
