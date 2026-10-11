//! Awakening Drum (TEF, ACE SPEC Item): draw a card for each of your Ancient Pokémon in play.
//!
//! One Draw event of the Item's effect. A card with no effect can't be played: with no Ancient Pokémon in play, or an empty
//! deck, it can't be (id925, id2231; JP FAQ Awakening Drum).
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
