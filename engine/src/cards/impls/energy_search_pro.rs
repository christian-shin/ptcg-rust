//! Energy Search Pro (SSP, ACE SPEC): search your deck for any number of
//! Basic Energy cards of different types, reveal them, and put them into
//! your hand. Then, shuffle your deck.
//!
//! Twinleaf: no preventDefault (the item is discarded right away); max =
//! number of distinct `provides[0]` among the deck's Basic Energy. In the
//! prompt callback: same-name check on the first two cards (throws),
//! SHOW_CARDS_TO_PLAYER, MOVE_CARDS, then (fixed in phase 4b: the callback
//! never called `next()`) the generator continues and shuffles the deck.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EnergySearchPro", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut seen: Vec<Option<CardType>> = Vec::new();
    for c in g.st.players[p].deck.iter() {
        let d = g.st.cdef(c);
        if d.is_energy() && d.energy_type == EnergyType::Basic as u8 {
            let t = d.provides.first().copied();
            if !seen.contains(&t) {
                seen.push(t);
            }
        }
    }
    let mut filter = Filter::super_type(SuperType::Energy);
    filter.energy_type = Some(EnergyType::Basic as u8);
    let mut opts = ChooseCardsOpts::new(0, seen.len() as u8, false);
    opts.different_types = true;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    let cards: Vec<CardId> = first.cards().to_vec();
    if cards.len() > 1 && g.st.cdef(cards[0]).name == g.st.cdef(cards[1]).name {
        bail!("CAN_ONLY_SELECT_TWO_DIFFERENT_ENERGY_TYPES");
    }
    show_cards_to_player(g, 1 - p, cards.len());
    move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    // ShuffleDeckPrompt directly (not the SHUFFLE_DECK prefab): no wait prompt
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
