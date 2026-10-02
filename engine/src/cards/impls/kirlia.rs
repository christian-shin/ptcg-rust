//! Kirlia (M1S / ASC): Call Sign - search your deck for up to 3 Pokémon and
//! put them into your hand, then shuffle. Psyshot - 30.
//!
//! Twinleaf: a ChooseCardsPrompt over the deck (Pokémon, min 0, max 3, no
//! cancel; nothing is revealed), MOVE_CARDS to the hand, then a bare
//! ShuffleDeckPrompt (no trailing wait).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Kirlia@MEG|ASC", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(
            g,
            p,
            "CHOOSE_CARD_TO_HAND",
            ListRef::Deck(p as u8),
            Filter::super_type(SuperType::Pokemon),
            ChooseCardsOpts::new(0, 3, false),
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
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
