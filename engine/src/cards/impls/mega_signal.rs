//! Mega Signal (MEG): search your deck for a Mega Evolution Pokémon ex,
//! reveal it, and put it into your hand. Then, shuffle your deck.
//!
//! Twinleaf: deck cards that aren't Pokémon ex with the Mega tag are
//! blocked; MOVE_CARDS runs even with nothing chosen; the reveal is shown
//! only when a card was taken; the final shuffle has no trailing wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaSignal", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut blocked = Blocked::default();
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if !d.is_pokemon() || !d.has_tag(tag::POKEMON_EX_LOWER) || !d.has_tag(tag::POKEMON_SV_MEGA) {
            blocked.push(i as u8);
        }
    }
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut opts = ChooseCardsOpts::new(0, 1, false);
    opts.blocked = blocked;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn shuffle(g: &mut Game, p: usize) {
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            if !cards.is_empty() {
                let id = g.player_id(1 - p);
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: CardFrame { stage: 2, ..f } });
                return Ok(());
            }
            shuffle(g, p);
            Ok(())
        }
        2 => {
            shuffle(g, p);
            Ok(())
        }
        _ => Ok(()),
    }
}
