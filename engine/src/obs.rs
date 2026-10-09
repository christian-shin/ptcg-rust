//! Fixed-size observation tensors for learning, built in Rust.
//!
//! The observation is from one player's point of view and contains only
//! information that player can see: their own hand, both boards, public zone
//! sizes and discard contents. Card identities are card-database indices + 1
//! (0 = empty) so a policy can embed them.

use crate::game::Game;
use crate::interface::{Opt, SelectData};
use crate::list::*;
use crate::state::*;
use crate::types::*;

pub const MAX_HAND: usize = 20;
pub const MAX_DISCARD: usize = 30;
pub const SLOT_FEATURES: usize = 16;
pub const SLOTS: usize = 9; // active + 8 bench
pub const GLOBAL_FEATURES: usize = 16;
pub const PLAYER_FEATURES: usize = 16 + SLOTS * SLOT_FEATURES + MAX_DISCARD;
pub const OBS_SIZE: usize = GLOBAL_FEATURES + 2 * PLAYER_FEATURES + MAX_HAND;
pub const OPTION_FEATURES: usize = 12;

fn card_code(g: &Game, c: CardId) -> f32 {
    (g.st.cards[c as usize].def as f32) + 1.0
}

fn slot_features(g: &Game, p: usize, s: SlotId, out: &mut [f32]) {
    let slot = g.st.slot(p, s);
    let top = g.st.slot_pokemon(p, s);
    if let Some(c) = top {
        let d = g.st.cdef(c);
        out[0] = card_code(g, c);
        out[1] = (d.hp + slot.hp_bonus) as f32 / 100.0;
        out[2] = slot.damage as f32 / 100.0;
        out[3] = g.st.slot_pokemons(p, s).len() as f32;
        // Energy by type (grass..metal), then other.
        for e in slot.energies.iter() {
            let t = g.st.cdef(e).provides.first().copied().unwrap_or(ct::COLORLESS) as usize;
            let k = if (1..=8).contains(&t) { 3 + t } else { 12 };
            out[k] += 1.0;
        }
        out[13] = slot.tools.len() as f32;
        let mut conds = 0u32;
        for c in slot.special_conditions.iter() {
            conds |= 1 << c;
        }
        out[14] = conds as f32;
        out[15] = (g.st.turn - slot.pokemon_played_turn) as f32;
    }
}

/// Observation for `viewer` (player index). Length [`OBS_SIZE`].
pub fn observe(g: &Game, viewer: usize) -> Vec<f32> {
    let mut v = vec![0f32; OBS_SIZE];
    let st = &g.st;
    v[0] = st.turn as f32;
    v[1] = (st.active_player as usize == viewer) as u8 as f32;
    v[2] = st.phase as u8 as f32;
    v[3] = st.stadium_card().map(|c| card_code(g, c)).unwrap_or(0.0);
    v[4] = (st.players[viewer].supporter_turn > 0) as u8 as f32;
    v[5] = (st.players[viewer].energy_played_turn == st.turn) as u8 as f32;
    v[6] = (st.players[viewer].retreated_turn == st.turn) as u8 as f32;
    for (k, p) in [viewer, 1 - viewer].into_iter().enumerate() {
        let base = GLOBAL_FEATURES + k * PLAYER_FEATURES;
        let pl = &st.players[p];
        v[base] = pl.deck.len() as f32;
        v[base + 1] = pl.hand.len() as f32;
        v[base + 2] = pl.discard.len() as f32;
        v[base + 3] = pl.prize_left() as f32;
        v[base + 4] = pl.lostzone.len() as f32;
        v[base + 5] = pl.bench.len() as f32;
        v[base + 6] = pl.used_vstar as u8 as f32;
        v[base + 7] = pl.used_gx as u8 as f32;
        let slots = pl.all_slots();
        for (i, s) in slots.iter().enumerate().take(SLOTS) {
            let o = base + 16 + i * SLOT_FEATURES;
            slot_features(g, p, *s, &mut v[o..o + SLOT_FEATURES]);
        }
        let d0 = base + 16 + SLOTS * SLOT_FEATURES;
        for (i, c) in pl.discard.iter().take(MAX_DISCARD).enumerate() {
            v[d0 + i] = card_code(g, c);
        }
    }
    let h0 = GLOBAL_FEATURES + 2 * PLAYER_FEATURES;
    for (i, c) in st.players[viewer].hand.iter().take(MAX_HAND).enumerate() {
        v[h0 + i] = card_code(g, c);
    }
    v
}

/// Integer features of one option: `[type, number, area, index, playerIndex,
/// inPlayArea, inPlayIndex, attackId, cardId+1, serial+1, selectType, context]`.
pub fn option_features(sel: &SelectData, o: &Opt) -> [i32; OPTION_FEATURES] {
    let f = |x: Option<u8>| x.map(|v| v as i32).unwrap_or(-1);
    [
        o.kind as i32,
        o.number.unwrap_or(-1),
        f(o.area),
        f(o.index),
        f(o.player_index),
        f(o.in_play_area),
        f(o.in_play_index),
        f(o.attack_id),
        o.card_id.map(|c| c as i32 + 1).unwrap_or(0),
        o.serial.map(|c| c as i32 + 1).unwrap_or(0),
        sel.select_type as i32,
        sel.context as i32,
    ]
}
