//! Poké Pad (ASC / MC): search your deck for a Pokémon that doesn't have a
//! Rule Box, reveal it, put it into your hand, then shuffle.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PokePad", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let mut opts = ChooseCardsOpts::new(0, 1, true);
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if !d.is_pokemon() || d.has_rule_box() {
            opts.blocked.push(i as u8);
        }
    }
    search_deck_for_pokemon_to_hand(g, p, Filter::none(), opts)
}

