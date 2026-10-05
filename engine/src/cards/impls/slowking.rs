//! Slowking (SCR): Seek Inspiration — discard the top card of your deck; if
//! it is a Pokémon without a Rule Box, choose 1 of its attacks and use it as
//! this attack (COPY_ATTACK_FROM_POKEMON_LIST, see `copy_attack.rs`).
//! Super Psy Bolt — 120.
//!
//! Fixed (phase 4b, R3): the attack choice can't be cancelled (it used to
//! allow it, so the copied attack could be skipped), and nothing is chosen
//! when every attack of the Pokémon is locked.
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
    // `pokemonInQuestion`: every discard entry that is this card (a
    // duplicated card instance appears once per entry).
    let matches: Vec<CardId> = g.st.players[p].discard.iter().filter(|&c| c == topdeck).collect();
    if matches.is_empty() {
        return Ok(());
    }
    // "Choose 1 of its attacks and use it": no cancel. Nothing is chosen when
    // every attack of the Pokémon is locked for the Active (no valid answer).
    let a = g.st.players[p].active;
    let locked = g.st.slot(p, a).cannot_use_attacks_next_turn;
    if d.attacks.iter().all(|at| locked.iter().any(|n| *n == at.name)) {
        return Ok(());
    }
    crate::copy_attack::copy_attack_from_pokemon_list(g, e, &matches, false)
}
