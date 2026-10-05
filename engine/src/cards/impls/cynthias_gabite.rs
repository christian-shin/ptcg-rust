//! Cynthia's Gabite (DRI): Champion's Call - once during your turn, search
//! your deck for a Cynthia's Pokémon, reveal it, put it into your hand, then
//! shuffle. Dragon Slice - 40.
//!
//! Twinleaf: ABILITY_USED runs before the prompt; the choice is min 0 / max 1
//! with no cancel (non-Cynthia's Pokémon blocked, works on an empty deck);
//! the reveal to the opponent and the ShuffleDeckPrompt are queued together;
//! the once-per-turn marker is only added in the shuffle callback.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "CynthiasGabite",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn champions_call() -> crate::markers::MarkerName {
    marker!("CHAMPIONS_CALL_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(champions_call(), me);
        }
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(champions_call(), me) {
            m.remove_from(champions_call(), me);
        }
        return Ok(());
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(champions_call(), me) {
            bail!("POWER_ALREADY_USED");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        ability_used(g, p, me);
        let mut blocked = Blocked::default();
        for (i, c) in g.st.players[p].deck.iter().enumerate() {
            let d = g.st.cdef(c);
            if d.is_pokemon() && !d.has_tag(tag::CYNTHIAS) {
                blocked.push(i as u8);
            }
        }
        let mut opts = ChooseCardsOpts::new(0, 1, false);
        opts.blocked = blocked;
        let mut filter = Filter::none();
        filter.super_type = Some(SuperType::Pokemon as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, opts, Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if !cards.is_empty() {
                move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
                show_cards_to_player(g, 1 - p, cards.len());
            }
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: CardFrame { stage: 2, ..f } });
            Ok(())
        }
        2 => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            g.st.players[p].marker.add(champions_call(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            Ok(())
        }
        _ => Ok(()),
    }
}
