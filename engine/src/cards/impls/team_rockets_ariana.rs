//! Team Rocket's Ariana (DRI): draw until you have 5 cards in hand, or 8 if
//! all of your Pokémon in play are Team Rocket's Pokémon.
//!
//! Twinleaf: sets `rocketSupporter` (not when used as the effect of an attack);
//! the draw is one MOVE_CARDS per card. Fixed (phase 4b, R7C): a card that would
//! draw nothing can't be played (rulings 851, 959).
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
