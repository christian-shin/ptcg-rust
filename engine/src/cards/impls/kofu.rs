//! Kofu (SCR): put 2 cards from your hand on the bottom of your deck in any
//! order, then draw 4. TEMPORARY test port (b13) so Crabominable's Food Prep
//! has Kofu cards in the discard pile.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Kofu", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    if g.st.players[p].hand.len() <= 2 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let temp = g.alloc_temp(&[]);
    let t = match temp {
        ListRef::Temp(i) => i,
        _ => unreachable!(),
    };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = t as i32;
    choose_cards(g, p, "CHOOSE_CARDS_TO_PUT_ON_BOTTOM_OF_THE_DECK", ListRef::Hand(p as u8), Filter::none(), ChooseCardsOpts::new(2, 2, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let temp = ListRef::Temp(f.a[1] as u8);
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Hand(p as u8), temp, &cards, me)?;
            let id = g.player_id(p);
            let mut nf = f;
            nf.stage = 2;
            g.prompt(id, "CHOOSE_CARDS_ORDER", PromptKind::OrderCards { cards: temp, allow_cancel: false }, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let o = match first {
                Res::Order(o) => o,
                _ => return Ok(()),
            };
            crate::game::apply_order(&mut g.temps[f.a[1] as usize], o.as_slice());
            let cards: Vec<CardId> = g.lst(temp).to_vec();
            move_cards(g, temp, ListRef::Deck(p as u8), &cards, me)?;
            let n = g.st.players[p].deck.len().min(4);
            move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), n, me)
        }
        _ => Ok(()),
    }
}
