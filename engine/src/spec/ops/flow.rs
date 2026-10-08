//! Control flow: May, If, Coin, Choose, loops, Fail, attack copying, ending
//! the turn or the game, Custom (vocabulary v1 "Operations", control flow).
//!
//! A nested list is entered with `Flow::Enter(sel)`; `child` names the list
//! for each selector, and `again` decides whether a finished list runs again
//! (loops count passes in `Frame::iter`).

use super::super::run::{Flow, Frame, CHOICE_NO, CHOICE_NONE, CHOICE_YES, SUB_COIN_SEQ};
use super::super::*;
use crate::game::{Game, R};
use crate::list::CardId;
use crate::game::CoinCb;
use crate::prefabs::*;
use crate::prompts::Res;
use crate::types::*;

/// "You may": ask `asker` when `when` holds (otherwise nothing is asked and
/// nothing happens); on yes run `yes`, on no run `no`.
pub struct MaySpec {
    pub asker: Who,
    pub when: Cond,
    pub msg: &'static str,
    pub yes: &'static [Step],
    pub no: &'static [Step],
}
pub struct IfSpec {
    pub cond: Cond,
    pub yes: &'static [Step],
    pub no: &'static [Step],
}
/// How many coins are flipped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoinMode {
    /// One coin: `heads` or `tails` runs.
    One,
    /// This many coins in a row (a sequence prompt); `then` runs afterwards and
    /// reads the heads with `Num::Heads`.
    Count(u8),
    /// Flip until tails; `then` runs afterwards (`Num::Heads` counts the heads).
    UntilTails,
}

/// Flip coin(s) (`flipper`), when `before` holds (otherwise nothing is flipped).
pub struct CoinSpec {
    pub flipper: Who,
    pub mode: CoinMode,
    pub before: Cond,
    pub heads: &'static [Step],
    pub tails: &'static [Step],
    pub then: &'static [Step],
}
impl CoinSpec {
    pub const DEFAULT: CoinSpec = CoinSpec { flipper: Who::Me, mode: CoinMode::One, before: Cond::True, heads: &[], tails: &[], then: &[] };
}
pub struct ChooseSpec {}
pub struct ForEachSpec {}
pub struct RepeatSpec {}
pub struct ParallelSpec {}
/// The use fails (`error`) unless the condition holds (an attack that can't be used, say).
pub struct FailSpec {
    pub unless: Cond,
    pub error: &'static str,
}
pub struct PickAttackSpec {}
/// Choose an attack of `from`'s Active Pokémon (when its card matches
/// `predicate` and has attacks) and use it as this attack, as a copy session
/// (`copy_attack.rs`): no cancel, `retries` attempts when a chosen attack
/// can't be used. Runs before the damage, with the attack itself.
pub struct CopyAttackSpec {
    pub from: Who,
    pub predicate: Pred,
    pub retries: u8,
}
pub struct EndTurnSpec {}
/// The game ends and `winner` wins.
pub struct EndGameSpec {
    pub winner: Who,
}
pub struct CustomSpec {}

pub(crate) fn exec(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
        Op::May(m) => {
            if let Some(a) = f.recorded(g, me) {
                return Ok(match a {
                    CHOICE_YES if !m.yes.is_empty() => Flow::Enter(0),
                    CHOICE_NO if !m.no.is_empty() => Flow::Enter(1),
                    _ => Flow::Next,
                });
            }
            if !cond_m(g, me, f, &m.when)? {
                return Ok(Flow::Next);
            }
            confirmation_prompt(g, f.who(m.asker), m.msg, f.cont(me, 1));
            Ok(Flow::Suspend)
        }
        Op::If(i) => {
            if cond_m(g, me, f, &i.cond)? {
                Ok(if i.yes.is_empty() { Flow::Next } else { Flow::Enter(0) })
            } else {
                Ok(if i.no.is_empty() { Flow::Next } else { Flow::Enter(1) })
            }
        }
        Op::Coin(c) => {
            if !cond_m(g, me, f, &c.before)? {
                return Ok(Flow::Next);
            }
            let p = f.who(c.flipper);
            match c.mode {
                CoinMode::One => {
                    g.coin_flip(p, CoinCb::Card { card: me, frame: f.frame_at(1) })?;
                }
                CoinMode::Count(n) => coin_flip_sequence(g, p, n, CoinCb::SequenceCard { card: me, frame: f.frame_at(SUB_COIN_SEQ) })?,
                CoinMode::UntilTails => coin_flip_sequence(g, p, 0, CoinCb::SequenceCard { card: me, frame: f.frame_at(SUB_COIN_SEQ) })?,
            }
            Ok(Flow::Suspend)
        }
        Op::Fail(x) => {
            if !cond_m(g, me, f, &x.unless)? {
                crate::bail!(x.error);
            }
            Ok(Flow::Next)
        }
        Op::CopyAttack(c) => {
            let p = f.who(c.from);
            let Some(card) = g.st.active_pokemon(p) else { return Ok(Flow::Next) };
            if !(pred(g, card, &Pred::HasAttacks) && pred(g, card, &c.predicate)) {
                return Ok(Flow::Next);
            }
            crate::copy_attack::copy_attack_from_pokemon_list_retries(g, f.eff, &[card], false, c.retries)?;
            Ok(Flow::Next)
        }
        Op::EndGame(e) => {
            let winner = if f.who(e.winner) == 0 { WINNER_P1 } else { WINNER_P2 };
            crate::engine::phase::end_game(g, winner);
            Ok(Flow::Next)
        }
        _ => unimplemented!("spec op not implemented yet (ops/flow.rs)"),
    }
}

/// A single coin flip came up heads or tails.
pub(crate) fn resume_coin(_g: &mut Game, _me: CardId, _f: &mut Frame, op: &Op, heads: bool) -> R<Flow> {
    match op {
        Op::Coin(c) => {
            let list = if heads { c.heads } else { c.tails };
            Ok(if list.is_empty() { Flow::Next } else { Flow::Enter(if heads { 0 } else { 1 }) })
        }
        _ => Ok(Flow::Next),
    }
}

pub(crate) fn resume(_g: &mut Game, _me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        // A coin sequence finished (the frame carries the heads).
        Op::Coin(c) if f.sub == SUB_COIN_SEQ => Ok(if c.then.is_empty() { Flow::Next } else { Flow::Enter(2) }),
        Op::May(m) => {
            if first.as_bool() {
                Ok(if m.yes.is_empty() { Flow::Next } else { Flow::Enter(0) })
            } else {
                Ok(if m.no.is_empty() { Flow::Next } else { Flow::Enter(1) })
            }
        }
        _ => Ok(Flow::Next),
    }
}

/// Step D: a "you may" is answered before the damage; its yes branch's own
/// choices are made then too.
pub(crate) fn choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
        Op::May(m) => {
            if !cond_m(g, me, f, &m.when)? {
                // Nothing to decide: the effect is not carried out.
                f.record(g, me, CHOICE_NONE);
                return Ok(Flow::Next);
            }
            confirmation_prompt(g, f.who(m.asker), m.msg, f.cont(me, 1));
            Ok(Flow::Suspend)
        }
        _ => Ok(Flow::Next),
    }
}

pub(crate) fn resume_choice(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::May(m) => {
            let yes = first.as_bool();
            f.record(g, me, if yes { CHOICE_YES } else { CHOICE_NO });
            Ok(if yes && !m.yes.is_empty() { Flow::Enter(0) } else { Flow::Next })
        }
        _ => Ok(Flow::Next),
    }
}

pub(crate) fn implied_ok(_g: &Game, _me: CardId, _f: &Frame, _op: &Op) -> bool {
    true
}

pub fn child(op: &Op, sel: u8) -> &'static [Step] {
    match (op, sel) {
        (Op::May(m), 0) => m.yes,
        (Op::May(m), _) => m.no,
        (Op::If(i), 0) => i.yes,
        (Op::If(i), _) => i.no,
        (Op::Coin(c), 0) => c.heads,
        (Op::Coin(c), 1) => c.tails,
        (Op::Coin(c), _) => c.then,
        _ => &[],
    }
}

pub(crate) fn again(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op) -> bool {
    false
}
