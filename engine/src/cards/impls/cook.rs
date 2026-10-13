//! Cook (FST): heal 70 damage from your Active Pokémon.
//!
//! One Heal event on your Active Pokémon. It can't be played while the Active Pokémon has no damage (a Trainer that can't have
//! any effect can't be played, id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cook",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(70), clear_conditions: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
