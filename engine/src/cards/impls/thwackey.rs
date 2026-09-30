//! Thwackey (TWM): Boom Boom Groove — once during your turn, if your Active
//! Pokémon has the Festival Lead Ability, search your deck for a card and put
//! it into your hand, then shuffle. Beat — 50.
//!
//! Twinleaf: Festival Lead is looked up by name on the Active's printed
//! powers; with no Active Pokémon the ability silently does nothing. The
//! marker is cleared when this card is played and at its owner's end of
//! turn. The final ShuffleDeckPrompt has no trailing wait.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "Thwackey",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::POWER]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn drum() -> crate::markers::MarkerName {
    marker!("BOOM_BOOM_DRUM_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(drum(), me);
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(drum(), me) {
            m.remove_from(drum(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(drum(), me) {
            bail!("POWER_ALREADY_USED");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        let active = match g.st.active_pokemon(p) {
            Some(c) => c,
            None => return Ok(()),
        };
        if !g.st.cdef(active).powers.iter().any(|pw| pw.name == "Festival Lead") {
            bail!("CANNOT_USE_POWER");
        }
        g.st.players[p].marker.add(drum(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        ability_used(g, p, me);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
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
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
