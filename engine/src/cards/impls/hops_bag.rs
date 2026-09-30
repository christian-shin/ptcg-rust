//! Hop's Bag (JTG): search your deck for up to 2 Basic Hop's Pokémon and put
//! them onto your Bench. Then, shuffle your deck.
//!
//! Twinleaf: deck cards without the Hop's tag are blocked (filter: Basic
//! Pokémon); each chosen card is played with a PlayPokemonFromDeckEffect into
//! the empty Bench slots taken before the prompt; the card then moves
//! supporter→discard before a wait-less shuffle.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HopsBag", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let open = empty_bench_slots(g, p);
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if open.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut blocked = Blocked::default();
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        if !g.st.cdef(c).has_tag(tag::HOPS) {
            blocked.push(i as u8);
        }
    }
    let max = open.len().min(2) as u8;
    let mut opts = ChooseCardsOpts::new(0, max, false);
    opts.blocked = blocked;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let mut packed: u32 = 0;
    for (i, s) in open.iter().enumerate() {
        packed |= (*s as u32) << (4 * i);
    }
    f.a[1] = packed as i32;
    f.a[2] = open.len() as i32;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Deck(p as u8), filter, opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let pu = p as u8;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    for (i, c) in cards.iter().enumerate() {
        if i >= f.a[2] as usize {
            bail!("TypeError: Cannot read properties of undefined");
        }
        let s = (((f.a[1] as u32) >> (4 * i)) & 0xF) as SlotId;
        g.run_fx(Effect::PlayPokemonFromDeck { p: pu, card: *c, target: SlotRef::new(p, s) })?;
    }
    move_cards(g, ListRef::Supporter(pu), ListRef::Discard(pu), &[me], me)?;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: pu });
    Ok(())
}
