//! Brock's Scouting (JTG): search your deck for up to 2 Basic Pokémon or 1
//! Evolution Pokémon, reveal them, put them into your hand, then shuffle.
//!
//! Twinleaf order kept: the card moves itself to the supporter pile; the
//! final ShuffleDeckPrompt has no trailing WaitPrompt. Fixed (R1-16, rulings
//! 779 and 851): it can't be played with an empty deck (CANNOT_PLAY_THIS_CARD,
//! before the card moves).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BrocksScouting", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

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
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let (mut basics, mut evolutions) = (0u8, 0u8);
    let mut blocked = Blocked::default();
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.is_pokemon() && d.stage == Stage::Basic as u8 {
            basics += 1;
        } else if d.is_pokemon() {
            evolutions += 1;
        } else {
            blocked.push(i as u8);
        }
    }
    let max_basics = basics.min(2);
    let max_evolutions = evolutions.min(1);
    let mut opts = ChooseCardsOpts::new(0, max_basics + max_evolutions, false);
    opts.blocked = blocked;
    opts.max_basics = Some(max_basics);
    opts.max_evolutions = Some(max_evolutions);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn shuffle(g: &mut Game, me: CardId, p: usize) {
    let mut f = CardFrame::at(3);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: f });
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            if !cards.is_empty() {
                let oid = g.player_id(1 - p);
                g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: CardFrame { stage: 2, ..f } });
                return Ok(());
            }
            shuffle(g, me, p);
            Ok(())
        }
        2 => {
            shuffle(g, me, p);
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
