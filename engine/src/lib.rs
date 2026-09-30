//! Pokémon TCG rules engine with Twinleaf parity.
//!
//! See `PLAN.md` at the repository root. The engine mirrors Twinleaf's store
//! (effects propagated to cards, prompts with continuations) with plain-data
//! state: a whole game, including pending prompts, is `Copy`.

pub mod canonical;
pub mod carddb;
pub mod cards;
pub mod effects;
pub mod energy;
pub mod engine;
pub mod game;
pub mod gen;
pub mod interface;
pub mod list;
pub mod markers;
pub mod options;
pub mod prompts;
pub mod rng;
pub mod state;
pub mod types;

pub use game::{Action, Game, GameError, Pending};
