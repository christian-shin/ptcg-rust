//! Applin (TWM 126): Find a Friend — search your deck for a Pokémon, reveal
//! it, put it into your hand, then shuffle. Rolling Tackle — 30.
//!
//! Twinleaf: the ShowCardsPrompt to the opponent is created even when no
//! card was chosen; the ShuffleDeckPrompt follows with no trailing wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Applin@Applin TWM2", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, source) = match *g.e(e) {
        Effect::Attack { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    let src = g.st.slot_pokemon(source.p as usize, source.s).unwrap_or(NO_CARD);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = src as i32;
    choose_cards(
        g,
        p,
        "CHOOSE_CARD_TO_HAND",
        ListRef::Deck(p as u8),
        Filter::super_type(SuperType::Pokemon),
        ChooseCardsOpts::new(0, 1, false),
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let src = f.a[1] as CardId;
    let first = results.first().copied().unwrap_or(Res::Null);
    let cards: Vec<CardId> = first.cards().to_vec();
    for c in &cards {
        move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &[*c], src)?;
    }
    let oid = g.player_id(1 - p);
    g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
