//! Slowpoke (PBL, Slowpoke M5): All-You-Can-Yeet — you may discard any number
//! of cards from your hand. Headbutt — 20.
//!
//! Twinleaf (pitch-black file): empty hand → nothing; ChooseCardsPrompt on
//! the hand (superType ANY), min 0, max hand size, no cancel; the chosen
//! cards go to the discard in one MOVE_CARDS.
//!
//! Twinleaf quirk kept: `matchesPromptFilter` compares `card.superType ===
//! SuperType.ANY` literally, so no card matches and only the empty
//! selection is possible — the attack never discards anything.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Slowpoke@PBL", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let max = g.st.players[p].hand.len();
    if max == 0 {
        return Ok(());
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(
        g,
        p,
        "CHOOSE_CARD_TO_DISCARD",
        ListRef::Hand(p as u8),
        Filter::super_type(SuperType::Any),
        ChooseCardsOpts::new(0, max as u8, false),
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if !cards.is_empty() {
        move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
    }
    Ok(())
}
