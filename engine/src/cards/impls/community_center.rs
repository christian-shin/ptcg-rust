//! Community Center (TWM, stadium): once during each player's turn, if that player has already played a Supporter from their
//! hand, they may heal 10 damage from each of their Pokémon.
//!
//! Using the Stadium needs a Supporter played this turn and a damaged Pokémon (a Stadium use that can change nothing isn't
//! allowed, id925); it heals each of the player's Pokémon on which Stadium effects are active, one Heal event each.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CommunityCenter",
    // Once during each player's turn, if that player has already played a Supporter from their
    // hand, they may heal 10 damage from each of their Pokémon (it can't be used without damage).
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[Cond::PlayedThisTurn(Who::Me, Pred::Supporter), Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::Damaged)],
        steps: &[Step::new(Op::ForEach(ForEachSpec {
            over: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::StadiumEffectActive),
            body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(10), clear_conditions: false }))],
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
