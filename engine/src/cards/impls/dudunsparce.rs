//! Dudunsparce (TEF): Run Away Draw — draw 3 cards, then shuffle this
//! Pokémon and all attached cards into your deck.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dudunsparce", mask: mask(&[k::POWER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_power_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Power { p, .. } => p as usize,
        _ => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_USE_POWER");
    }
    move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 3, me)?;
    for (s, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if c != me {
            continue;
        }
        move_pokemon_off_board(g, SlotRef::new(p, s), ListRef::Deck(p as u8), me)?;
        // Plain ShuffleDeckPrompt (no SHUFFLE_DECK animation wait).
        let id = g.player_id(p);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        if let Some(Res::Order(o)) = results.first() {
            crate::game::apply_order(&mut g.st.players[f.a[0] as usize].deck, o.as_slice());
        }
    }
    Ok(())
}
