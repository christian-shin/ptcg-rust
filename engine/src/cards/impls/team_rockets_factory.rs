//! Team Rocket's Factory (DRI, stadium): once during each player's turn, if
//! they played a "Team Rocket" Supporter from their hand this turn, they may
//! draw 2 cards.
//!
//! Twinleaf reads `player.rocketSupporter` (set by Team Rocket's Petrel);
//! the FACTORY_USED_MARKER is only bookkeeping (cleared at end of turn).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsFactory",
    // Once during each player's turn, if they played a Team Rocket's Supporter from their hand this turn, they may draw 2 cards.
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[Cond::RocketSupporterPlayed(Who::Me), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
