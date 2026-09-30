//! Sacred Ash (FLF): shuffle 5 Pokémon from your discard pile into your deck.
//!
//! Twinleaf: `min = max = min(5, Pokémon in discard)`, cancellable (a cancel
//! ends the effect; the card is still cleaned up as played); the final
//! ShuffleDeckPrompt has no trailing wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SacredAsh", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let n = g.st.players[p].discard.iter().filter(|c| g.st.cdef(*c).is_pokemon()).count();
    if n == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let max = n.min(5) as u8;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(
        g,
        p,
        "CHOOSE_CARD_TO_DECK",
        ListRef::Discard(p as u8),
        Filter::super_type(SuperType::Pokemon),
        ChooseCardsOpts::new(max, max, true),
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            if cards.is_empty() {
                return Ok(());
            }
            move_cards(g, ListRef::Discard(p as u8), ListRef::Deck(p as u8), &cards, me)?;
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            if let Some(Res::Order(o)) = results.first() {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
