//! Buddy-Buddy Poffin (TEF): search your deck for up to 2 Basic Pokémon with
//! 70 HP or less and put them onto your Bench, then shuffle.
//!
//! Twinleaf's ShuffleDeckPrompt here has no trailing WaitPrompt.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BuddyBuddyPoffin", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let open = empty_bench_slots(g, p);
    if g.st.players[p].deck.is_empty() || open.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut opts = ChooseCardsOpts::new(0, 0, false);
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if !(d.is_pokemon() && d.stage == Stage::Basic as u8 && d.hp <= 70) {
            opts.blocked.push(i as u8);
        }
    }
    let max = open.len().min(2) as u8;
    opts.max = max;
    opts.max_pokemons = Some(max);
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    for (i, s) in open.iter().enumerate().take(3) {
        f.a[1 + i] = *s as i32;
    }
    f.l[0] = open.len() as u8;
    let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Deck(p as u8), filter, opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            for (i, c) in cards.iter().enumerate() {
                // openSlots[index]; at most 2 cards are selectable.
                if i >= f.l[0] as usize || i >= 3 {
                    bail!("TypeError: Cannot read properties of undefined");
                }
                let s = f.a[1 + i] as SlotId;
                g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, s) })?;
            }
            move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)?;
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

