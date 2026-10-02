//! Drayton (SSP): look at the top 7 cards of your deck. Choose a Pokémon and
//! a Trainer card from those cards, reveal them, and put them into your
//! hand. Shuffle the other cards back into your deck.
//!
//! Twinleaf: throws on an empty deck; no Supporter-already-played check; the
//! card moves to the Supporter area and the play is prevented; 7 cards go to a
//! temporary list in which Energy cards are blocked, with `maxTrainers` and
//! `maxPokemons` of min(count, 1) each (different types required); the chosen
//! cards go to the hand, the rest to the bottom of the deck; ShowCards for the
//! opponent only when something was taken; a final wait-less shuffle.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Drayton", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn move_all(g: &mut Game, src: ListRef, dst: ListRef, me: CardId) -> R {
    g.run_fx(Effect::MoveCards { source: src, destination: dst, cards: None, count: None, to_top: false, to_bottom: false, skip_cleanup: false, source_card: me })?;
    Ok(())
}

fn shuffle_nowait(g: &mut Game, p: usize) {
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let temp = g.alloc_temp(&[]);
    move_count_from(g, ListRef::Deck(p as u8), temp, 7, me)?;

    let (mut trainers, mut pokemons) = (0u8, 0u8);
    let mut opts = ChooseCardsOpts::new(0, 0, false);
    for (i, c) in g.lst(temp).iter().enumerate() {
        let d = g.st.cdef(*c);
        if d.is_trainer() {
            trainers += 1;
        } else if d.is_pokemon() {
            pokemons += 1;
        } else {
            opts.blocked.push(i as u8);
        }
    }
    let max_trainers = trainers.min(1);
    let max_pokemons = pokemons.min(1);
    opts.max = max_trainers + max_pokemons;
    opts.max_trainers = Some(max_trainers);
    opts.max_pokemons = Some(max_pokemons);
    opts.allow_different_super_types = true;
    opts.different_types = true;
    let t = match temp {
        ListRef::Temp(i) => i,
        _ => unreachable!(),
    };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = t as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", temp, Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let temp = ListRef::Temp(f.a[1] as u8);
    match f.stage {
        1 => {
            let chosen: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            move_cards(g, temp, ListRef::Hand(p as u8), &chosen, me)?;
            move_all(g, temp, ListRef::Deck(p as u8), me)?;
            if !chosen.is_empty() {
                let id = g.player_id(1 - p);
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
                return Ok(());
            }
            shuffle_nowait(g, p);
            Ok(())
        }
        2 => {
            shuffle_nowait(g, p);
            Ok(())
        }
        _ => Ok(()),
    }
}
