//! Lasting effects and markers: attack flags, "during your opponent's next
//! turn" effects (vocabulary v1 "Operations", state; "Core trackers").

use super::super::run::{Flow, Frame};
use super::super::*;
use crate::game::{Game, R};
use crate::list::CardId;
use crate::prompts::Res;

pub struct AttackFlagSpec {}
pub struct SetMarkerSpec {}
pub struct ClearMarkerSpec {}
pub struct ArmSpec {}

pub(crate) fn exec(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op) -> R<Flow> {
    unimplemented!("spec op not implemented yet (ops/state.rs)")
}

pub(crate) fn resume(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op, _results: &[Res]) -> R<Flow> {
    Ok(Flow::Next)
}
