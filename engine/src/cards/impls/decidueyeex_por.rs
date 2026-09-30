//! Decidueye ex (POR / M3): Sniper's Eye — if your opponent has exactly 4
//! cards in their hand, ignore all [C] Energy in the costs of attacks used by
//! this Pokémon. Crushing Arrow — 240; discard an Energy from your
//! opponent's Active Pokémon.
//!
//! Twinleaf quirks kept: Sniper's Eye strips every [C] from any
//! CheckAttackCostEffect while this card is the player's Active Pokémon, with
//! no ability-lock check. Crushing Arrow (AFTER_ATTACK) returns when the
//! opponent's Active `energies` list is empty, otherwise a non-cancellable
//! ChooseCardsPrompt (Energy) over the Active's cards and MOVE_CARDS to the
//! discard pile.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Decidueyeex@POR", mask: mask(&[k::CHECK_ATTACK_COST, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckAttackCost { p, .. } = *g.e(e) {
        let p = p as usize;
        if g.st.slot_pokemon(p, g.st.players[p].active) == Some(me) && g.st.players[1 - p].hand.len() == 4 {
            if let Effect::CheckAttackCost { cost, .. } = g.e_mut(e) {
                cost.retain(|t| *t != ct::COLORLESS);
            }
        }
        return Ok(());
    }

    if after_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::AfterAttack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let o = 1 - p;
        let oa = g.st.players[o].active;
        if g.st.slot(o, oa).energies.is_empty() {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_CARD_TO_DISCARD",
            PromptKind::ChooseCards { cards: ListRef::Slot(o as u8, oa), filter: Filter::super_type(SuperType::Energy), opts: ChooseCardsOpts::new(1, 1, false) },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    let selected: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if selected.is_empty() {
        return Ok(());
    }
    let oa = g.st.players[o].active;
    move_cards(g, ListRef::Slot(o as u8, oa), ListRef::Discard(o as u8), &selected, me)
}
