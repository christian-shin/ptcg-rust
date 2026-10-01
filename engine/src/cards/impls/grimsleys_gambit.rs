//! Grimsley's Move (PFL / M2, class GrimsleysGambit): look at the top 7
//! cards of your deck and put a [D] Pokémon you find there onto your Bench;
//! shuffle the other cards and put them on the bottom of your deck. Can't be
//! used on your first turn.
//!
//! Twinleaf: no move to the supporter pile and no preventDefault (the core
//! discards the Supporter normally); checks in order: Supporter played, empty
//! deck, full Bench, turn 1/2. The ShuffleDeckPrompt targets the (already
//! emptied) top-cards list, so its order is never applied to anything and the
//! other cards go to the bottom unshuffled. The Supporter pile → discard move
//! runs before the prompt answers, and the new Pokémon gets
//! `pokemonPlayedTurn = turn` (no PlayPokemonEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GrimsleysGambit", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let open = empty_bench_slots(g, p);
    if open.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if g.st.turn == 1 || g.st.turn == 2 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let top = g.alloc_temp(&[]);
    move_count(g, ListRef::Deck(p as u8), top, 7)?;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), card_type: Some(ct::DARK), card_type_list: true, ..Filter::none() };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.l[0] = match top {
        ListRef::Temp(i) => i,
        _ => 0,
    };
    f.l[1] = open.as_slice()[0];
    choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", top, filter, ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let top = ListRef::Temp(f.l[0]);
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    for c in cards.iter() {
        let slot = f.l[1];
        move_cards(g, top, ListRef::Slot(p as u8, slot), &[*c], NO_CARD)?;
        g.st.players[p].slots[slot as usize].pokemon_played_turn = g.st.turn;
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Noop);
    g.run_fx(Effect::MoveCards {
        source: top,
        destination: ListRef::Deck(p as u8),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: true,
        skip_cleanup: false,
        source_card: NO_CARD,
    })?;
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], NO_CARD)?;
    Ok(())
}
