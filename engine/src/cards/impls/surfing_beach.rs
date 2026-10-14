//! Surfing Beach (CRI 114, Stadium): once during each player's turn, that player may switch their Active [W] Pokémon with 1 of
//! their Benched [W] Pokémon.
//!
//! A Stadium use (once per player turn by the Stadium's use rule). It needs an Active [W] Pokémon and a Benched [W] Pokémon;
//! the Benched [W] Pokémon is chosen, then the switch is a ChangeActive (Switch, APR C-03).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SurfingBeach",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        // An Active [W] Pokémon and a Benched [W] Pokémon.
        needs: &[
            Cond::Slot(MY_ACTIVE, SlotPred::TypeIs(ct::WATER)),
            Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::TypeIs(ct::WATER)),
        ],
        steps: &[
            Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::TypeIs(ct::WATER)) }, purpose: PromptPurpose::SwitchPokemon })),
            Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Picked, required: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
