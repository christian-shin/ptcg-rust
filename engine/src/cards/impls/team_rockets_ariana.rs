//! Team Rocket's Ariana (DRI): draw cards until you have 5 cards in your hand. If all of your Pokémon in play are Team Rocket's
//! Pokémon, draw cards until you have 8 cards in your hand instead.
//!
//! One Draw (a Draw event carrying the cards, D3). A card that would draw nothing can't be used (usability needs a card in the
//! deck). Played from the hand it is the turn's Team Rocket's Supporter that Team Rocket's Factory reads
//! (`Cond::PlayedThisTurn`); used through Look-Alike Show it isn't played (D5).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsAriana",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        // Draw until you have 5 cards in hand, or 8 if all of your Pokémon in play are Team Rocket's Pokémon.
        steps: &[Step::new(Op::Draw(DrawSpec {
            who: Who::Me,
            amount: DrawAmount::UntilHandSize(Num::If(&Cond::AllSlots(SlotSel::Pokemon(Who::Me), SlotPred::Top(Pred::Tag(tag::TEAM_ROCKET))), &Num::Lit(8), &Num::Lit(5))),
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
