//! Bug Catching Set (TWM): look at the top 7 cards of your deck; you may
//! reveal up to 2 [G] Pokémon / Basic [G] Energy there and put them into your
//! hand, then shuffle the other cards back into your deck.
//!
//! Twinleaf quirks kept: `max` counts matching cards in the whole deck, the
//! remaining cards are put on the bottom of the deck, and when nothing is
//! taken the deck is not shuffled.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BugCatchingSet", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn matches(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    (d.is_pokemon() && d.card_type.contains(&ct::GRASS)) || (d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.name == "Grass Energy")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let deck: Vec<CardId> = g.st.players[p].deck.iter().collect();
    let mut count = 0u8;
    let mut blocked: Vec<usize> = Vec::new();
    for (i, c) in deck.iter().enumerate() {
        if matches(g, *c) {
            count += 1;
        } else {
            blocked.push(i);
        }
    }
    let max = count.min(2);
    let temp = g.alloc_temp(&[]);
    g.run_fx(Effect::MoveCards {
        source: ListRef::Deck(p as u8),
        destination: temp,
        cards: None,
        count: Some(7),
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    // The constructor keeps the blocked indices that address cards of the new list.
    let mut opts = ChooseCardsOpts::new(0, max, false);
    let n = g.lst(temp).len();
    for i in blocked {
        if i < n {
            opts.blocked.push(i as u8);
        }
    }
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

fn move_all(g: &mut Game, src: ListRef, dst: ListRef, me: CardId) -> R {
    g.run_fx(Effect::MoveCards { source: src, destination: dst, cards: None, count: None, to_top: false, to_bottom: false, skip_cleanup: false, source_card: me })?;
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let temp = ListRef::Temp(f.a[1] as u8);
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if cards.is_empty() {
                return move_all(g, temp, ListRef::Deck(p as u8), me);
            }
            move_cards(g, temp, ListRef::Hand(p as u8), &cards, me)?;
            move_all(g, temp, ListRef::Deck(p as u8), me)?;
            let id = g.player_id(1 - p);
            let mut nf = f;
            nf.stage = 2;
            g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let id = g.player_id(p);
            let mut nf = f;
            nf.stage = 3;
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        3 => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
