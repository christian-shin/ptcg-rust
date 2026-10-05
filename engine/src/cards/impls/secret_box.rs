//! Secret Box (TWM, ACE SPEC): discard 3 other cards from your hand; search
//! your deck for an Item, a Pokémon Tool, a Supporter and a Stadium, reveal
//! them, put them into your hand, then shuffle.
//!
//! Twinleaf quirks kept: a MOVE_CARDS hand->supporter of the card (already in
//! the supporter pile) is reduced; the final ShuffleDeckPrompt has no trailing
//! WaitPrompt. Fixed (phase 4b, R3): choosing nothing from the deck no longer
//! skips the shuffle.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SecretBox", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let hand: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
    if hand.len() < 3 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    // Fixed (phase 4b, rulings 779/851/1098): a search of an empty deck is not possible, so the card can't be played.
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let hand: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
    let temp = g.alloc_temp(&hand);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_CARD_TO_DISCARD",
        PromptKind::ChooseCards { cards: temp, filter: Filter::none(), opts: ChooseCardsOpts::new(3, 3, false) },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn shuffle(g: &mut Game, me: CardId, p: usize) {
    let mut f = CardFrame::at(4);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: f });
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            // Deck counts: taken before the discard in Twinleaf (the deck is unchanged by it).
            let (mut tools, mut items, mut stadiums, mut supporters) = (0u8, 0u8, 0u8, 0u8);
            let mut blocked = Blocked::default();
            for (i, c) in g.st.players[p].deck.iter().enumerate() {
                let d = g.st.cdef(c);
                match (d.is_trainer(), d.trainer_type()) {
                    (true, TrainerType::Tool) => tools += 1,
                    (true, TrainerType::Item) => items += 1,
                    (true, TrainerType::Stadium) => stadiums += 1,
                    (true, TrainerType::Supporter) => supporters += 1,
                    _ => blocked.push(i as u8),
                }
            }
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
            let (mt, mi, ms, msu) = (tools.min(1), items.min(1), stadiums.min(1), supporters.min(1));
            let mut opts = ChooseCardsOpts::new(0, mt + mi + ms + msu, false);
            opts.blocked = blocked;
            opts.max_tools = Some(mt);
            opts.max_items = Some(mi);
            opts.max_stadiums = Some(ms);
            opts.max_supporters = Some(msu);
            choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: CardFrame { stage: 2, ..f } });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if cards.is_empty() {
                shuffle(g, me, p);
                return Ok(());
            }
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            let oid = g.player_id(1 - p);
            g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: CardFrame { stage: 3, ..f } });
            Ok(())
        }
        3 => {
            shuffle(g, me, p);
            Ok(())
        }
        4 => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
