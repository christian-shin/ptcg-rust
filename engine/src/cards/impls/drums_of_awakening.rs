//! Awakening Drum (TEF, ACE SPEC): draw a card for each of your Ancient
//! Pokémon in play.
//!
//! Twinleaf: one MOVE_CARDS deck→hand with `count`; since phase 4b the card is
//! unplayable with no Ancient Pokémon in play or an empty deck (rulings 851/1733).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DrumsOfAwakening",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[Step::new(Op::Draw(DrawSpec {
            who: Who::Me,
            amount: DrawAmount::Count(Num::InPlayCount(Who::Me, PlayScope::All, Pred::Tag(crate::types::tag::ANCIENT))),
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
