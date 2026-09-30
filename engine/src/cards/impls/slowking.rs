//! Slowking (SCR): Seek Inspiration — discard the top card of your deck; if
//! it is a Pokémon without a Rule Box, choose 1 of its attacks and use it as
//! this attack (COPY_ATTACK_FROM_POKEMON_LIST, see `copy_attack.rs`).
//! Super Psy Bolt — 120.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Slowking",
    mask: mask(&[k::ATTACK]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        return Ok(());
    }
    let top = g.alloc_temp(&[]);
    move_count_from(g, ListRef::Deck(p as u8), top, 1, me)?;
    let topdeck = g.lst(top).first().copied();
    g.run_fx(Effect::MoveCards {
        source: top,
        destination: ListRef::Discard(p as u8),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    let topdeck = match topdeck {
        Some(c) => c,
        None => return Ok(()),
    };
    let d = g.st.cdef(topdeck);
    if !d.is_pokemon() || d.has_rule_box() {
        return Ok(());
    }
    if !g.st.players[p].discard.iter().any(|c| c == topdeck) {
        return Ok(());
    }
    crate::copy_attack::copy_attack_from_pokemon_list(g, e, &[topdeck], true)
}
