//! Arven's Sandwich (DRI 161, Item): heal 30 damage from your Active Pokémon. If that Pokémon is an Arven's Pokémon, heal 100
//! damage from it instead.
//!
//! One Heal event on the Active Pokémon, its amount 100 when the Active Pokémon carries the "Arven's" name tag, otherwise 30.
//! It can't be played when the Active Pokémon has no damage (a Trainer that can change nothing: id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ArvensSandwich",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::Heal(HealSpec {
            target: SlotTarget::Slot(MY_ACTIVE),
            hp: Num::If(&Cond::Slot(MY_ACTIVE, SlotPred::Tag(crate::types::tag::ARVENS)), &Num::Lit(100), &Num::Lit(30)),
            clear_conditions: false,
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
