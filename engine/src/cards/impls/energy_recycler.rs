//! Energy Recycler (BST): shuffle up to 5 basic Energy cards from your
//! discard pile into your deck.
//!
//! Twinleaf: the prompt requires at least 1 card (min 1, max 5); the final
//! ShuffleDeckPrompt has no animation wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EnergyRecycler", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut count = 0;
    // Twinleaf computes a `blocked` list but never passes it to the prompt.
    let opts = ChooseCardsOpts::new(1, 5, false);
    for c in g.st.players[p].discard.iter() {
        let d = g.st.cdef(c);
        if d.is_energy() && d.energy_type == EnergyType::Basic as u8 {
            count += 1;
        }
    }
    if count == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_DECK", ListRef::Discard(p as u8), filter, opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Discard(p as u8), ListRef::Deck(p as u8), &cards, me)?;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
