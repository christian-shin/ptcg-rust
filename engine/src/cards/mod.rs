//! Card behaviors. Each Twinleaf card class with logic is ported as a
//! [`CardImpl`]: a `reduce` handler for the effects it subscribes to, plus
//! resume functions for its continuations (Twinleaf generators/closures).
//!
//! The registry maps a behavior class name (`CardDef::behavior`) to its
//! implementation, so reprints share one port.

use crate::carddb::{cards as all_defs, DefId};
use crate::effects::{EffId, KindMask};
use crate::game::{Game, R};
use crate::list::CardId;
use crate::prompts::Res;
use std::sync::OnceLock;

/// Continuation payload for card generators and callbacks.
#[derive(Clone, Copy, Debug, Default)]
pub struct CardFrame {
    pub stage: u8,
    pub a: [i32; 4],
    pub e: [EffId; 2],
    pub l: [u8; 2],
}

impl CardFrame {
    pub fn at(stage: u8) -> CardFrame {
        CardFrame { stage, ..Default::default() }
    }
}

pub type ReduceFn = fn(&mut Game, CardId, EffId) -> R;
pub type ResumeFn = fn(&mut Game, CardId, CardFrame, &[Res]) -> R;
pub type CoinFn = fn(&mut Game, CardId, CardFrame, bool) -> R;
pub type CanPlayFn = fn(&mut Game, CardId, usize) -> bool;

pub struct CardImpl {
    /// Twinleaf behavior class name, or `Class@Full Name` to bind one card
    /// only (for distinct Twinleaf classes that share a name).
    pub class: &'static str,
    /// Effect kinds the handler reacts to (bit = `Effect::kind()`).
    pub mask: KindMask,
    pub reduce: ReduceFn,
    pub resume: Option<ResumeFn>,
    pub coin: Option<CoinFn>,
    pub can_play: Option<CanPlayFn>,
}

pub mod registry;

/// Everything a card port usually needs.
pub mod prelude {
    pub use super::{CardFrame, CardImpl};
    pub use crate::bail;
    pub use crate::effects::{k, mask, EffId, Effect, SlotRef, PowerRef, AtkBase};
    pub use crate::game::{CoinCb, Cont, Game, GameError, R};
    pub use crate::list::*;
    pub use crate::prefabs::*;
    pub use crate::prompts::*;
    pub use crate::state::*;
    pub use crate::types::*;
}

fn table() -> &'static Vec<Option<&'static CardImpl>> {
    static T: OnceLock<Vec<Option<&'static CardImpl>>> = OnceLock::new();
    T.get_or_init(|| {
        all_defs()
            .iter()
            .map(|d| {
                if d.behavior.is_empty() {
                    None
                } else {
                    registry::IMPLS.iter().copied().find(|i| class_matches(i.class, d))
                }
            })
            .collect()
    })
}

/// `class` is the Twinleaf class name, optionally qualified as `Class@SET`
/// when two Twinleaf files define classes with the same name (e.g.
/// `Koraidonex` in both ASC and TEF).
fn class_matches(class: &str, d: &crate::carddb::CardDef) -> bool {
    match class.split_once('@') {
        // `Class@SET` or `Class@Full Name` pins a port to one printing;
        // `Class@A|B` to several (e.g. an old printing and its reprint).
        Some((c, q)) => c == d.behavior && q.split('|').any(|q| q == d.tl_set || q == d.tl_full_name),
        None => class == d.behavior,
    }
}

#[inline]
pub fn impl_for(d: DefId) -> Option<&'static CardImpl> {
    table()[d as usize]
}

/// The declarative spec of a converted card (PLAN.md 8.5), matched like its `CardImpl`.
pub fn spec_for(d: DefId) -> Option<&'static crate::spec::CardSpec> {
    static T: OnceLock<Vec<Option<&'static crate::spec::CardSpec>>> = OnceLock::new();
    T.get_or_init(|| {
        all_defs()
            .iter()
            .map(|d| {
                if d.behavior.is_empty() {
                    None
                } else {
                    registry::SPECS.iter().copied().find(|s| class_matches(s.class, d))
                }
            })
            .collect()
    })[d as usize]
}

/// Cards whose Twinleaf class has logic but no port yet.
pub fn missing_behavior(d: DefId) -> bool {
    !all_defs()[d as usize].behavior.is_empty() && impl_for(d).is_none()
}

pub fn resume(g: &mut Game, card: CardId, f: CardFrame, results: &[Res]) -> R {
    let d = g.st.cards[card as usize].def;
    match impl_for(d).and_then(|i| i.resume) {
        Some(r) => r(g, card, f, results),
        None => Ok(()),
    }
}

pub fn coin_result(g: &mut Game, card: CardId, f: CardFrame, result: bool) -> R {
    let d = g.st.cards[card as usize].def;
    match impl_for(d).and_then(|i| i.coin) {
        Some(r) => r(g, card, f, result),
        None => Ok(()),
    }
}
