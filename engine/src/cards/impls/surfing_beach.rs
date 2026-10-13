//! Surfing Beach (CRI 114, Stadium): once during each player's turn, that player may switch their Active [W] Pokémon with 1 of
//! their Benched [W] Pokémon.
//!
//! A Stadium use (once per player turn by the Stadium's use rule). It needs an Active [W] Pokémon, a Benched [W] Pokémon and the
//! Stadium's effect not blocked on the Active Pokémon; the Benched [W] Pokémon is chosen, then the switch is a ChangeActive (Switch,
//! APR C-03).
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
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Picked, msg: "", required: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
