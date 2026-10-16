//! Fixed-size observation tensors for learning, built in Rust.
//!
//! The observation is from one player's point of view and contains only
//! information that player can see: their own hand, both boards, public zone
//! sizes and discard contents. Card identities are card-database indices + 1
//! (0 = empty) so a policy can embed them.

use crate::game::Game;
use crate::interface::{Opt, SelectData};
use crate::view::{CardView, PokemonView, TableView};

pub const MAX_HAND: usize = 20;
pub const MAX_DISCARD: usize = 30;
pub const SLOT_FEATURES: usize = 16;
pub const SLOTS: usize = 9; // active + 8 bench
pub const GLOBAL_FEATURES: usize = 16;
pub const PLAYER_FEATURES: usize = 16 + SLOTS * SLOT_FEATURES + MAX_DISCARD;
pub const OBS_SIZE: usize = GLOBAL_FEATURES + 2 * PLAYER_FEATURES + MAX_HAND;
pub const OPTION_FEATURES: usize = 12;

fn card_code(c: &CardView) -> f32 {
    c.def as f32 + 1.0
}

fn slot_features(turn: i32, pk: &PokemonView, out: &mut [f32]) {
    out[0] = card_code(&pk.top);
    out[1] = pk.max_hp as f32 / 100.0;
    out[2] = pk.damage as f32 / 100.0;
    out[3] = (1 + pk.pre_evolution.len()) as f32;
    // Energy by type (grass..metal), then other.
    for e in pk.energy.iter() {
        let t = e.first_type as usize;
        let k = if (1..=8).contains(&t) { 3 + t } else { 12 };
        out[k] += 1.0;
    }
    out[13] = pk.tools.len() as f32;
    out[14] = pk.conditions as f32;
    out[15] = (turn - pk.entered_turn) as f32;
}

/// Observation for `viewer` (player index), read from the [`TableView`] of the viewer. Length [`OBS_SIZE`].
pub fn observe(g: &Game, viewer: usize) -> Vec<f32> {
    observe_view(&TableView::new(g, viewer))
}

/// The features of a view.
pub fn observe_view(t: &TableView) -> Vec<f32> {
    let mut v = vec![0f32; OBS_SIZE];
    v[0] = t.turn as f32;
    v[1] = (t.active_player == t.viewer) as u8 as f32;
    v[2] = t.phase as f32;
    v[3] = t.stadium.as_ref().map(|(c, _)| card_code(c)).unwrap_or(0.0);
    v[4] = t.me().supporter_played as u8 as f32;
    v[5] = t.me().energy_attached as u8 as f32;
    v[6] = t.me().retreated as u8 as f32;
    for (k, pl) in [t.me(), t.opponent()].into_iter().enumerate() {
        let base = GLOBAL_FEATURES + k * PLAYER_FEATURES;
        v[base] = pl.deck_count as f32;
        v[base + 1] = pl.hand_count as f32;
        v[base + 2] = pl.discard.len() as f32;
        v[base + 3] = pl.prize_count as f32;
        v[base + 4] = pl.lost_zone.len() as f32;
        v[base + 5] = pl.bench_max as f32;
        v[base + 6] = pl.used_vstar as u8 as f32;
        v[base + 7] = pl.used_gx as u8 as f32;
        let spots = std::iter::once(&pl.active).chain(pl.bench.iter().take(pl.bench_max as usize));
        for (i, pk) in spots.enumerate().take(SLOTS) {
            if let Some(pk) = pk {
                let o = base + 16 + i * SLOT_FEATURES;
                slot_features(t.turn, pk, &mut v[o..o + SLOT_FEATURES]);
            }
        }
        let d0 = base + 16 + SLOTS * SLOT_FEATURES;
        for (i, c) in pl.discard.iter().take(MAX_DISCARD).enumerate() {
            v[d0 + i] = card_code(c);
        }
    }
    let h0 = GLOBAL_FEATURES + 2 * PLAYER_FEATURES;
    if let Some(hand) = &t.me().hand {
        for (i, c) in hand.iter().take(MAX_HAND).enumerate() {
            v[h0 + i] = card_code(c);
        }
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

#[cfg(test)]
mod tests {
    //! The observation read from the `TableView` equals the features the state gave directly (the code before C13b, kept
    //! here as the reference), on many states of many games.
    use super::*;
    use crate::list::*;
    use crate::state::*;
    use crate::types::*;

fn card_code(g: &Game, c: crate::list::CardId) -> f32 {
        (g.st.cards[c as usize].def as f32) + 1.0
    }
    
    fn slot_features(g: &Game, facts: &mut crate::legal::Ctx, p: usize, s: SlotId, out: &mut [f32]) {
        let slot = g.st.slot(p, s);
        let top = g.st.slot_pokemon(p, s);
        if let Some(c) = top {
            let d = g.st.cdef(c);
            out[0] = card_code(g, c);
            // The HP with its modifiers (the derived layer over the game, no memo).
            out[1] = crate::derived::compute_hp(facts, p, s).unwrap_or(d.hp) as f32 / 100.0;
            out[2] = slot.damage as f32 / 100.0;
            out[3] = g.st.slot_pokemons(p, s).len() as f32;
            // Energy by type (grass..metal), then other.
            for e in slot.energies().iter() {
                let t = g.st.cdef(e).provides.first().copied().unwrap_or(ct::COLORLESS) as usize;
                let k = if (1..=8).contains(&t) { 3 + t } else { 12 };
                out[k] += 1.0;
            }
            out[13] = slot.tools().len() as f32;
            let mut conds = 0u32;
            for c in slot.special_conditions.iter() {
                conds |= 1 << c;
            }
            out[14] = conds as f32;
            out[15] = (g.st.turn - slot.entered_turn) as f32;
        }
    }
    
    /// Observation for `viewer` (player index). Length [`OBS_SIZE`].
    pub fn observe_before(g: &Game, viewer: usize) -> Vec<f32> {
        let mut v = vec![0f32; OBS_SIZE];
        let mut facts = crate::legal::Ctx::new(g);
        let st = &g.st;
        v[0] = st.turn as f32;
        v[1] = (st.active_player as usize == viewer) as u8 as f32;
        v[2] = st.phase as u8 as f32;
        v[3] = st.stadium_card().map(|c| card_code(g, c)).unwrap_or(0.0);
        v[4] = st.players[viewer].played_this_turn.iter().any(|c| st.cdef(*c).trainer_type == crate::types::TrainerType::Supporter as u8) as u8 as f32;
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
                // The opponent's Pokémon put down face down at setup are hidden (batch 10 E12).
                if p != viewer && st.in_play_face_down() {
                    continue;
                }
                let o = base + 16 + i * SLOT_FEATURES;
                slot_features(g, &mut facts, p, *s, &mut v[o..o + SLOT_FEATURES]);
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
    
    
    #[test]
    fn the_view_gives_the_features_of_the_state() {
        let mut checked = 0;
        for seed in 0..12u32 {
            let mut names: Vec<&str> = Vec::new();
            for (n, k) in [("Duraludon PRE 69", 4), ("Team Rocket's Mewtwo ex DRI 81", 3), ("Sacred Charm PFL 93", 4), ("Fire Energy MEE 2", 12)] {
                names.extend(std::iter::repeat(n).take(k));
            }
            while names.len() < 60 {
                names.push("Metal Energy MEE 8");
            }
            let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
            let mut g = Game::new(seed);
            g.start([&deck, &deck]).unwrap();
            g.settle().unwrap();
            for step in 0..400 {
                let Some(sel) = g.select_with(false).unwrap() else { break };
                for viewer in 0..2 {
                    assert_eq!(observe(&g, viewer), observe_before(&g, viewer), "seed {seed} step {step} viewer {viewer}");
                    checked += 1;
                }
                let mut picks = Vec::new();
                while picks.len() < sel.min_count {
                    let (mask, _) = g.pick_mask(&sel, &picks);
                    let allowed: Vec<usize> = (0..mask.len()).filter(|j| mask[*j]).collect();
                    picks.push(allowed[(seed as usize + step) % allowed.len()]);
                }
                if g.answer(&sel, &picks).is_err() {
                    break;
                }
            }
        }
        assert!(checked > 1000);
    }
}
