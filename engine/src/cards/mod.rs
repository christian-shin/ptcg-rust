//! Card behaviors. Each card with logic is a [`CardImpl`]: a `reduce` handler for the effects it subscribes to, plus
//! resume functions for its continuations.
//!
//! The registry maps a behavior class key (`CardDef::behavior`, an official card name; see
//! tools/rename_classes.py) to its implementation, so reprints share one port.

use crate::carddb::{cards as all_defs, DefId};
use crate::effects::{EffId, KindMask};
use crate::game::{Game, R};
use crate::list::CardId;
use crate::prompts::Res;
use std::sync::OnceLock;

/// Continuation payload for card routines that wait for an answer, and callbacks.
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
    /// Behavior class name, or `Class@SET` / `Class@Full Name` to bind one
    /// printing only (for distinct classes that share a name).
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

/// The declarative spec of a converted card, matched like its `CardImpl`.
pub fn spec_for(d: DefId) -> Option<&'static crate::spec::CardSpec> {
    static T: OnceLock<Vec<Option<&'static crate::spec::CardSpec>>> = OnceLock::new();
    T.get_or_init(|| {
        all_defs()
            .iter()
            .map(|d| {
                if d.behavior.is_empty() {
                    None
                } else {
                    registry::SPECS.iter().copied().find(|s| s.class == d.behavior)
                }
            })
            .collect()
    })[d as usize]
}

/// Cards whose behavior class has logic but no port yet.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every card resolves to the implementation it resolved to before the behavior class keys were renamed to
    /// official names (tools/rename_classes.py): `impl_table.txt` was written from the old matching (`Class@SET`
    /// qualifiers and all) before the rename. A key names exactly one implementation, and a card with a behavior
    /// the table lists as "-" has none.
    #[test]
    fn every_def_resolves_to_the_impl_it_had_before_the_rename() {
        let mut want: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
        for line in include_str!("impl_table.txt").lines().filter(|l| !l.starts_with('#')) {
            let (card, key) = line.split_once('\t').unwrap();
            assert!(want.insert(card, key).is_none(), "{card} twice");
        }
        let mut keys = std::collections::HashSet::new();
        for i in registry::IMPLS {
            assert!(keys.insert(i.class), "two implementations carry the key {}", i.class);
        }
        for (id, d) in all_defs().iter().enumerate() {
            let got = impl_for(id as DefId).map(|i| i.class);
            match want.get(d.full_name) {
                Some(&"-") => assert!(got.is_none() && !d.behavior.is_empty(), "{}", d.full_name),
                Some(&key) => {
                    assert_eq!(d.behavior, key, "{}: behavior key", d.full_name);
                    assert_eq!(got, Some(key), "{}: implementation", d.full_name);
                }
                // A card ported after the rename isn't in the table: its behavior key is the key of its implementation.
                None => assert!(d.behavior.is_empty() == got.is_none() && got.map_or(true, |class| class == d.behavior), "{}: not in the frozen table", d.full_name),
            }
        }
    }
}
