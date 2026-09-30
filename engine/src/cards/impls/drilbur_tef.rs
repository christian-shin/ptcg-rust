//! Drilbur (TEF): Dig Dig Dig — when you play this Pokémon from your hand
//! onto your Bench during your turn, you may search your deck for up to 3
//! Basic [F] Energy cards and discard them, then shuffle. Sand Spray — 20.
//!
//! Twinleaf: on its PlayPokemonEffect (returns early on an empty deck or a
//! blocked ability, checked while the card is still in hand) a Confirm
//! prompt; yes → ChooseCardsPrompt (min 0, max 3, no cancel, basic energy
//! named "Fighting Energy"); choosing nothing ends it without a shuffle;
//! otherwise MOVE_CARDS deck→discard and a bare ShuffleDeckPrompt.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Drilbur@TEF", mask: mask(&[k::PLAY_POKEMON]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card != me {
            return Ok(());
        }
        let p = p as usize;
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if !first.as_bool() {
                return Ok(());
            }
            let mut filter = Filter::super_type(SuperType::Energy);
            filter.energy_type = Some(EnergyType::Basic as u8);
            filter.name = Some("Fighting Energy");
            let mut nf = f;
            nf.stage = 2;
            choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 3, false), Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if cards.is_empty() {
                return Ok(());
            }
            move_cards(g, ListRef::Deck(p as u8), ListRef::Discard(p as u8), &cards, me)?;
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
            Ok(())
        }
        _ => Ok(()),
    }
}
