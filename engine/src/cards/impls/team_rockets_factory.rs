//! Team Rocket's Factory (DRI, Stadium): once during each player's turn, if they played a Supporter card that has "Team Rocket" in
//! its name from their hand this turn, they may draw 2 cards.
//!
//! The condition is `Cond::PlayedThisTurn(Who::Me, Supporter & Team Rocket's)` (the turn's play record from PlayTrainer; a Supporter
//! used through Look-Alike Show isn't played, D5). One Draw of 2 cards; once per player turn by the Stadium's use rule.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsFactory",
    // Once during each player's turn, if they played a Team Rocket's Supporter from their hand this turn, they may draw 2 cards.
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[Cond::PlayedThisTurn(Who::Me, Pred::All(&[Pred::Supporter, Pred::Tag(crate::types::tag::TEAM_ROCKET)]))],
        steps: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
