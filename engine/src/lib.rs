//! Pokémon TCG rules engine with Twinleaf parity.
//!
//! See docs/ENGINE.md (local, not in git). The engine mirrors Twinleaf's store
//! (effects propagated to cards, prompts with continuations) with plain-data
//! state: a whole game, including pending prompts, is `Copy`.

pub mod canonical;
pub mod cause;
pub mod spec;
pub mod carddb;
pub mod copy_attack;
pub mod cards;
pub mod effects;
pub mod energy;
pub mod engine;
pub mod game;
pub mod gen;
pub mod interface;
pub mod invariants;
pub mod legal;
pub mod legal_stats;
pub mod list;
pub mod markers;
pub mod options;
pub mod prefabs;
pub mod prompts;
pub mod obs;
pub mod rng;
pub mod expect;
pub mod scenario;
pub mod search;
pub mod selfplay;
pub mod state;
pub mod types;

pub use game::{Action, Game, GameError, Pending};
