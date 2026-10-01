//! Frogadier (CRI, Frogadier M4): Summoning Jutsu — search your deck for up
//! to 3 Pokémon, reveal them, and put them into your hand. Then, shuffle your
//! deck. Aqua Edge — 50.
//!
//! Twinleaf (chaos-rising file): no Pokémon in the deck → nothing happens
//! (no shuffle); ChooseCardsPrompt min 0, max min(3, Pokémon in deck), no
//! cancel; one MOVE_CARDS per chosen card (sourceCard = the attacking
//! Pokémon); no reveal prompt; a final ShuffleDeckPrompt with no wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Frogadier@Frogadier M4", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, source) = match *g.e(e) {
        Effect::Attack { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    let n = g.st.players[p].deck.iter().filter(|c| g.st.cdef(*c).is_pokemon()).count();
    let max = n.min(3) as u8;
    if max == 0 {
        return Ok(());
    }
    let src_card = g.st.slot_pokemon(source.p as usize, source.s).unwrap_or(NO_CARD);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = src_card as i32;
    choose_cards(
        g,
        p,
        "CHOOSE_CARD_TO_HAND",
        ListRef::Deck(p as u8),
        Filter::super_type(SuperType::Pokemon),
        ChooseCardsOpts::new(0, max, false),
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            let src = f.a[1] as CardId;
            for c in cards {
                move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &[c], src)?;
            }
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
