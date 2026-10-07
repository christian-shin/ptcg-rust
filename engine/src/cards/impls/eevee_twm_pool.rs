//! Eevee (TWM 135): Ascension — search your deck for a card that evolves from
//! this Pokémon and put it onto this Pokémon to evolve it, then shuffle.
//! Quick Attack — 20+; flip a coin, if heads 20 more damage.
//!
//! Twinleaf: an empty deck skips everything; with no evolution in the deck only
//! SHUFFLE_DECK runs; else a non-cancellable ChooseCardsPrompt ({ superType:
//! POKEMON }, min 0, max 1) with every non-evolution index blocked. The
//! evolution is a plain MOVE_CARDS onto the slot holding this card, then
//! `clearEffects()`, `pokemonPlayedTurn = turn`, SHUFFLE_DECK.
use super::riolu_pre::{coin_more_damage, flip_more_damage};
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EeveeTWMPool", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let name = g.st.cdef(me).name;
        let mut opts = ChooseCardsOpts::new(0, 1, false);
        let mut all_blocked = true;
        for (i, c) in g.st.players[p].deck.iter().enumerate() {
            let d = g.st.cdef(c);
            if d.is_pokemon() && d.evolves_from == name {
                all_blocked = false;
            } else {
                opts.blocked.push(i as u8);
            }
        }
        if all_blocked {
            shuffle_deck(g, p);
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_EVOLVE", ListRef::Deck(p as u8), Filter::super_type(SuperType::Pokemon), opts, Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        return flip_more_damage(g, me, e, 20);
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if !cards.is_empty() {
        let (q, s) = match g.st.locate(me) {
            Some(ListRef::Slot(q, s)) => (q, s),
            _ => bail!("INVALID_GAME_STATE"),
        };
        move_cards(g, ListRef::Deck(p as u8), ListRef::Slot(q, s), &cards[..1], me)?;
        let turn = g.st.turn;
        let slot = &mut g.st.players[q as usize].slots[s as usize];
        crate::engine::game_effect::clear_effects(slot);
        slot.pokemon_played_turn = turn;
    }
    shuffle_deck(g, p);
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    coin_more_damage(g, f, heads)
}
