//! Dragon Elixir (SSP 172, Item): heal 60 damage from your Active [N] Pokémon.
//!
//! One Heal event of the Active Pokémon, which must be [N] as the game checks its type (an effect that changes the type
//! counts): the type is the card's own condition on its target, an effect possibility the rules' walk can't read from the
//! program (a checked read), so `needs` holds it. It can't be played when the Active Pokémon isn't [N] or has no damage (a
//! Trainer that can change nothing: id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DragonElixir",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Slot(MY_ACTIVE, SlotPred::TypeIs(ct::DRAGON))],
        steps: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(60), clear_conditions: false }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
