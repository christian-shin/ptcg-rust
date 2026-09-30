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
    /// Twinleaf behavior class name.
    pub class: &'static str,
    /// Effect kinds the handler reacts to (bit = `Effect::kind()`).
    pub mask: KindMask,
    pub reduce: ReduceFn,
    pub resume: Option<ResumeFn>,
    pub coin: Option<CoinFn>,
    pub can_play: Option<CanPlayFn>,
}

pub mod registry;

fn table() -> &'static Vec<Option<&'static CardImpl>> {
    static T: OnceLock<Vec<Option<&'static CardImpl>>> = OnceLock::new();
    T.get_or_init(|| {
        all_defs()
            .iter()
            .map(|d| {
                if d.behavior.is_empty() {
                    None
                } else {
                    registry::IMPLS.iter().copied().find(|i| i.class == d.behavior)
                }
            })
            .collect()
    })
}

#[inline]
pub fn impl_for(d: DefId) -> Option<&'static CardImpl> {
    table()[d as usize]
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
