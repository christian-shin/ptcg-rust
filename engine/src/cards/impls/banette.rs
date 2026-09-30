//! Banette (PBL / M5): Hide 'n' Sneak. Puppet Pull — 80; you may search
//! your deck for a card and put it into your hand, then shuffle.
//!
//! Twinleaf: AFTER_ATTACK opens a ConfirmPrompt (WANT_TO_DRAW_CARDS); on yes
//! with a non-empty deck, SEARCH_DECK_FOR_CARDS_TO_HAND with no filter
//! (min 1, max 1, no cancel; no reveal).
use super::shuppet::{reduce_hide_n_sneak, HIDE_N_SNEAK_KINDS};
use crate::cards::prelude::*;
use crate::effects::KindMask;

const MASK: KindMask = mask(&HIDE_N_SNEAK_KINDS) | mask(&[k::AFTER_ATTACK]);

pub static IMPL: CardImpl = CardImpl { class: "Banette", mask: MASK, reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    reduce_hide_n_sneak(g, me, e)?;
    if after_attack_used(g, e, 0, me) {
        if let Effect::AfterAttack { p, .. } = *g.e(e) {
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            confirmation_prompt(g, p as usize, "WANT_TO_DRAW_CARDS", Cont::Card { card: me, frame: f });
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let yes = results.first().map(|r| r.as_bool()).unwrap_or(false);
    if yes && !g.st.players[p].deck.is_empty() {
        search_deck_for_cards_to_hand(g, p, me, Filter::none(), ChooseCardsOpts::new(1, 1, false));
    }
    Ok(())
}
