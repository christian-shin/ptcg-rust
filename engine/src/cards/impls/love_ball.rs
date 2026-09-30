//! Love Ball (TWM): search your deck for a Pokémon with the same name as 1
//! of your opponent's Pokémon in play, reveal it, and put it into your hand.
//! Then, shuffle your deck.
//!
//! Twinleaf quirks kept: no preventDefault; `opponent.bench.filter(card
//! instanceof PokemonCard)` is always empty (bench entries are card lists),
//! so only the name of `opponent.active.cards[0]` (the bottom card) is
//! allowed; the choice is min 1 / max 1 and can be cancelled; MOVE_CARDS
//! runs even with nothing chosen, then the reveal, then the shuffle.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LoveBall", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let o = 1 - p;
    let active = g.st.players[o].active;
    let allowed = match g.st.slot(o, active).cards.iter().next() {
        Some(c) => g.st.cdef(c).name,
        None => bail!("TypeError: Cannot read properties of undefined (reading 'name')"),
    };
    let mut blocked = Blocked::default();
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.is_pokemon() && d.name != allowed {
            blocked.push(i as u8);
        }
    }
    let mut opts = ChooseCardsOpts::new(1, 1, true);
    opts.blocked = blocked;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::super_type(SuperType::Pokemon), opts, Cont::Card { card: me, frame: f });
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
                let id = g.player_id(1 - p);
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: CardFrame { stage: 2, ..f } });
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
