//! Gimmighoul (SSP): Minor Errand-Running - search your deck for up to 2
//! Basic Energy cards, reveal them, put them into your hand, shuffle. Tackle - 50.
//!
//! Twinleaf: the ShuffleDeckPrompt is created right after the
//! ChooseCardsPrompt (before it is answered), with no trailing wait; the
//! ShowCards info prompt (only when any cards were chosen) is created in the
//! choose callback.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Gimmighoul", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Default::default() };
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 2, false), Cont::Card { card: me, frame: f });
        let id = g.player_id(p);
        g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    let cards: Vec<CardId> = first.cards().to_vec();
    show_cards_to_player(g, 1 - p, cards.len());
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    Ok(())
}
