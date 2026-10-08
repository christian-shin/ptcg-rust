//! Community Center (TWM, stadium): once during each player's turn, if that
//! player has already played a Supporter from their hand, they may heal 10
//! damage from each of their Pokémon.
//!
//! Twinleaf: throws when `supporterTurn === 0`; otherwise a HealEffect(10)
//! on each of the player's Pokémon whose stadium effect isn't blocked
//! (undamaged Pokémon included). Fixed in phase 4b (R4, Rulings Compendium 851/1830):
//! it also throws when no Pokémon has damage (no effect, so it can't be used).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CommunityCenter",
    // Once during each player's turn, if that player has already played a Supporter from their
    // hand, they may heal 10 damage from each of their Pokémon (it can't be used without damage).
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[Cond::SupporterPlayed(Who::Me), Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::Damaged)],
        steps: &[Step::new(Op::ForEach(ForEachSpec {
            over: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::StadiumEffectActive),
            body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(10), via: HealVia::Effect, clear_conditions: false }))],
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
