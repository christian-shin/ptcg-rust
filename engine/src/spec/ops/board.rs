//! Damage, damage counters, healing, switching, Special Conditions, Knock
//! Outs, evolution, removing Pokémon from play (vocabulary v1 "Operations",
//! board and slots).

use super::super::run::{Flow, Frame};
use super::super::*;
use crate::game::{Game, R};
use crate::list::CardId;
use crate::prefabs::*;
use crate::prompts::Res;

pub struct PickSlotSpec {}
pub struct SwitchSpec {}
pub struct HealSpec {}
pub struct DamageSpec {}
pub struct DamageSlotSpec {}
pub struct PlaceCountersSpec {}
pub struct SpreadCountersSpec {}
pub struct MoveCountersSpec {}
pub struct EvolveSpec {}
pub struct DevolveSpec {}
pub struct SwapPokemonCardSpec {}
/// Put a Pokémon and all cards attached to it into a zone.
pub struct RemoveFromPlaySpec {
    pub slot: SlotExpr,
    pub destination: ZoneRef,
}
pub struct ConditionsSpec {}
pub struct KnockOutSpec {}

pub(crate) fn exec(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
        Op::RemoveFromPlay(r) => {
            if let Some(slot) = slot_of(g, me, f, r.slot) {
                let dst = zone_ref(f, r.destination);
                move_pokemon_off_board(g, slot, dst, me)?;
            }
            Ok(Flow::Next)
        }
        _ => unimplemented!("spec op not implemented yet (ops/board.rs)"),
    }
}

pub(crate) fn resume(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op, _results: &[Res]) -> R<Flow> {
    Ok(Flow::Next)
}

pub(crate) fn choice(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op) -> R<Flow> {
    Ok(Flow::Next)
}

pub(crate) fn resume_choice(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op, _results: &[Res]) -> R<Flow> {
    Ok(Flow::Next)
}

pub(crate) fn implied_ok(_g: &Game, _me: CardId, _f: &Frame, _op: &Op) -> bool {
    true
}
