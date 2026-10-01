//! Precious Trolley (SSP, ACE SPEC): search your deck for any number of Basic
//! Pokémon and put them onto your Bench. Then, shuffle your deck.
//!
//! Twinleaf: the card moves hand→supporter and the TrainerEffect is
//! prevented before the empty-deck / full-bench checks (so a failed play
//! still throws after the move, which the rollback undoes). The prompt takes
//! 0..open-slots Basic Pokémon (no cancel); each goes to the next empty slot
//! via PlayPokemonFromDeckEffect, then the card moves supporter→discard and
//! a bare ShuffleDeckPrompt (no wait) follows.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PreciousTrolley", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let slots = empty_bench_slots(g, p);
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if slots.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
    choose_cards(
        g,
        p,
        "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
        ListRef::Deck(p as u8),
        filter,
        ChooseCardsOpts::new(0, slots.len() as u8, false),
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    // `slots` was captured before the prompt; nothing fills the bench meanwhile.
    let slots = empty_bench_slots(g, p);
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    for (i, c) in cards.iter().enumerate() {
        let s = match slots.get(i) {
            Some(s) => *s,
            None => bail!("TypeError: Cannot read properties of undefined"),
        };
        g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, s) })?;
    }
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)?;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
