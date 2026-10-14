//! Fennel (BLK 82 / SV11B): Supporter. Heal 40 damage from each of your Pokémon.
//!
//! One Heal event per Pokémon, Active first, then the Bench. It can't be played when no Pokémon of yours has damage (a card
//! with no effect can't be played: id925; the rules' walk of the program finds it, so the card declares no condition).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Fennel",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::ForEach(ForEachSpec { over: SlotSel::Pokemon(Who::Me), body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(40), clear_conditions: false }))] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
