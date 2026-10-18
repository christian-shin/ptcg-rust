//! Pokémon TCG rules engine.
//!
//! See docs/ENGINE.md (local, not in git). Effects are propagated to cards and prompts carry continuations, over
//! plain-data state: a whole game, including pending prompts, is `Copy`.

pub mod canonical;
pub mod cause;
pub mod spec;
pub mod carddb;
pub mod copy_attack;
pub mod decl;
pub mod derived;
pub mod cards;
pub mod effects;
pub mod energy;
pub mod engine;
pub mod game;
pub mod gen;
pub mod host;
pub mod interface;
pub mod invariants;
pub mod legal;
pub mod legal_stats;
pub mod list;
pub mod options;
pub mod place;
pub mod record;
pub mod prefabs;
pub mod prompts;
pub mod purpose;
pub mod obs;
pub mod rng;
pub mod expect;
pub mod scenario;
pub mod search;
pub mod selfplay;
pub mod state;
pub mod table_rules;
pub mod types;
pub mod view;
pub mod what_if;
#[cfg(test)]
mod api_tests;

pub use game::{Action, Game, GameError, Pending};
