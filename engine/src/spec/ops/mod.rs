//! Op families and the interpreter's dispatch to them. Each family file owns
//! its ops' record types and implements, per op:
//!
//! - `exec`: run the op (or its after-damage half for an attack);
//! - `resume`: continue after the op's own prompt;
//! - `choice` / `resume_choice`: its step-D half for an attack (ask and
//!   record the choice before the damage), when it has one;
//! - `implied_ok`: the preconditions it implies for a Trainer or an Ability.
//!
//! Flow ops also give their nested lists (`child`) and loop control (`again`).

pub mod board;
pub mod cards;
pub mod flow;
pub mod state;

use super::run::{Flow, Frame};
use super::*;
use crate::game::{Game, R};
use crate::list::CardId;
use crate::prompts::Res;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Family {
    Cards,
    Board,
    Flow,
    State,
}

fn family(op: &Op) -> Family {
    use Op::*;
    match op {
        Discard(_) | PutIntoHand(_) | PutIntoDeck(_) | Look(_) | Pick(_) | Draw(_) | Shuffle(_) | Reveal(_) | Search(_) | Snapshot(_) | Attach(_) | MoveEnergy(_) | DiscardEnergy(_)
        | PlayFromZone(_) | PickPrize(_) | PrizeVisibility(_) | TakePrize(_) | HandShuffleDraw(_) | PlayAsPokemon(_) | MoveEnergyOwn(_) | PrizeBonus(_) | BotherBot(_) | UseTrainer(_) => Family::Cards,
        PickSlot(_) | Switch(_) | Heal(_) | Damage(_) | DamageSlot(_) | PlaceCounters(_) | SpreadCounters(_) | MoveCounters(_) | Evolve(_)
        | Devolve(_) | SwapPokemonCard(_) | RemoveFromPlay(_) | Conditions(_) | KnockOut(_) | SpreadDamage(_) | SwitchWithActive(_) | EachSlot(_) | ChoiceDamage(_) => Family::Board,
        Coin(_) | May(_) | If(_) | Choose(_) | ForEach(_) | Repeat(_) | Parallel(_) | Fail(_) | PickAttack(_) | CopyAttack(_) | EndTurn(_)
        | EndGame(_) => Family::Flow,
        AttackFlag(_) | SetMarker(_) | ClearMarker(_) | Arm(_) | ArmEach(_) | AbilityUsed(_) => Family::State,
    }
}

pub(crate) fn exec(g: &mut Game, me: CardId, f: &mut Frame, op: &'static Op) -> R<Flow> {
    match family(op) {
        Family::Cards => cards::exec(g, me, f, op),
        Family::Board => board::exec(g, me, f, op),
        Family::Flow => flow::exec(g, me, f, op),
        Family::State => state::exec(g, me, f, op),
    }
}

pub(crate) fn resume(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    match family(op) {
        Family::Cards => cards::resume(g, me, f, op, results),
        Family::Board => board::resume(g, me, f, op, results),
        Family::Flow => flow::resume(g, me, f, op, results),
        Family::State => state::resume(g, me, f, op, results),
    }
}

pub(crate) fn choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match family(op) {
        Family::Cards => cards::choice(g, me, f, op),
        Family::Board => board::choice(g, me, f, op),
        Family::Flow => flow::choice(g, me, f, op),
        Family::State => Ok(Flow::Next),
    }
}

pub(crate) fn resume_choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    match family(op) {
        Family::Cards => cards::resume_choice(g, me, f, op, results),
        Family::Board => board::resume_choice(g, me, f, op, results),
        Family::Flow => flow::resume_choice(g, me, f, op, results),
        Family::State => Ok(Flow::Next),
    }
}

pub(crate) fn implied_ok(g: &Game, me: CardId, f: &Frame, op: &Op) -> bool {
    match family(op) {
        Family::Cards => cards::implied_ok(g, me, f, op),
        Family::Board => board::implied_ok(g, me, f, op),
        Family::Flow => flow::implied_ok(g, me, f, op),
        Family::State => true,
    }
}

/// Would the op change the game now (`Some`), or is it not an effect by itself (`None`: a choice the following ops act
/// on, a marker, a flag)? The rules' usability (`run::usable_by_rules`) needs one effect that would change the game.
pub(crate) fn effect_possible(g: &Game, me: CardId, f: &Frame, op: &Op) -> Option<bool> {
    match family(op) {
        Family::Cards => cards::effect_possible(g, me, f, op),
        Family::Board => board::effect_possible(g, me, f, op),
        Family::Flow => flow::effect_possible(g, me, f, op),
        Family::State => None,
    }
}

/// Does the condition read what the last op did ("If you do")?
pub(crate) fn reads_done(c: &Cond) -> bool {
    flow::reads_done(c)
}

/// A Draw after `removed` cards left the player's hand earlier in the program (`run::usable_by_rules`).
pub(crate) fn draw_possible_after(g: &Game, me: CardId, f: &Frame, d: &DrawSpec, removed: i32) -> bool {
    cards::draw_possible_after(g, me, f, d, removed)
}

/// Would the pick find a card (`run::usable_by_rules`)?
pub(crate) fn pick_finds(g: &Game, me: CardId, f: &Frame, p: &PickSpec) -> bool {
    cards::pick_finds(g, me, f, p)
}

/// Any op of the list would change the game (`None` when none is an effect).
pub(crate) fn any_effect_possible(g: &Game, me: CardId, f: &Frame, steps: &[Step]) -> Option<bool> {
    let mut out = None;
    for s in steps {
        match effect_possible(g, me, f, &s.op) {
            Some(true) => return Some(true),
            Some(false) => out = Some(false),
            None => {}
        }
    }
    out
}

/// Does the condition of a `Fail` op hold (read by the executor and by legality)?
pub(crate) fn fail_holds<F: crate::derived::Facts>(g: &mut F, me: CardId, f: &Frame, x: &FailSpec) -> R<bool> {
    flow::fail_holds(g, me, f, x)
}

/// The nested list `sel` of a flow op (`&[]` for ops without one).
pub fn child(op: &Op, sel: u8) -> &'static [Step] {
    flow::child(op, sel)
}

/// After a nested list of `op` ends: run it again (a loop's next pass)?
pub(crate) fn again(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> bool {
    flow::again(g, me, f, op)
}

/// A finished coin sequence of a `Coin` op: `results` are the bits (bit i = flip i heads) and
/// the flip count.
pub(crate) fn resume_coin(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    flow::resume_coin(g, me, f, op, results)
}

/// A single flip of a `Coin` op came up `heads` (or tails).
pub(crate) fn coin_result(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, heads: bool) -> R<Flow> {
    flow::coin_result(g, me, f, op, heads)
}
