//! Gumshoos (M1L): Gather Evidence — once during your turn, switch a card
//! from your hand with the top card of your deck. Bite — 50.
//!
//! Twinleaf: CANNOT_USE_POWER with an empty deck or hand, POWER_ALREADY_USED
//! with the marker, then a cancellable ChooseCardsPrompt on the hand. On a
//! pick: MOVE_CARDS deck→hand (count 1), then the picked card is spliced out
//! of the hand and unshifted onto the deck directly, marker + ABILITY_USED.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Gumshoos",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn gather() -> crate::markers::MarkerName {
    crate::marker!("GATHER_EVIDENCE_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(gather(), me);
        }
    }
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() || g.st.players[p].hand.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(gather(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DECK", ListRef::Hand(p as u8), Filter::none(), ChooseCardsOpts::new(1, 1, true), Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].marker.remove_from(gather(), me);
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let picked = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if picked.is_empty() {
        return Ok(());
    }
    move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 1, me)?;
    let c = picked[0];
    let pl = &mut g.st.players[p];
    if let Some(i) = pl.hand.index_of(c) {
        pl.hand.remove_at(i);
        pl.deck.insert(0, c);
    }
    g.st.players[p].marker.add(gather(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    ability_used(g, p, me);
    Ok(())
}
