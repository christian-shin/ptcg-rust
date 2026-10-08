//! Control flow: May, If, Coin, Choose, loops, Fail, attack copying, ending
//! the turn or the game, Custom (vocabulary v1 "Operations", control flow).
//!
//! A nested list is entered with `Flow::Enter(sel)`; `child` names the list
//! for each selector, and `again` decides whether a finished list runs again
//! (loops count passes in `Frame::iter`).

use super::super::run::{Flow, Frame, CHOICE_NO, CHOICE_NONE, CHOICE_YES};
use super::super::*;
use crate::game::{Game, R};
use crate::list::CardId;
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
    /// One flip: `heads` or `tails` runs.
    Single,
    /// This many flips, then `then` runs (`Num::Heads` counts the heads).
    Count(u8),
    /// Flip until tails, then `then` runs.
    UntilTails,
}

/// "Flip a coin(s)": the flipper flips, then the branch for the result runs.
pub struct CoinSpec {
    pub flipper: Who,
    pub mode: CoinMode,
    pub heads: &'static [Step],
    pub tails: &'static [Step],
    /// After the flips of a sequence.
    pub then: &'static [Step],
}
impl CoinSpec {
    pub const DEFAULT: CoinSpec = CoinSpec { flipper: Who::Me, mode: CoinMode::Single, heads: &[], tails: &[], then: &[] };
}
pub struct ChooseSpec {}
/// Run `body` once for each selected Pokémon, in order; inside the body the Pokémon is the
/// picked slot (`SlotExpr::Picked`). The list is read when the loop starts and again at each
/// pass by position.
pub struct ForEachSpec {
    pub over: SlotSel,
    pub body: &'static [Step],
}
pub struct RepeatSpec {}
pub struct ParallelSpec {}
pub struct FailSpec {}
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
        Op::ForEach(fe) => {
            let slots = slots_m(g, me, f, &fe.over)?;
            match slots.as_slice().first() {
                Some(s) => {
                    f.slot = s.p << 4 | s.s;
                    Ok(Flow::Enter(0))
                }
                None => Ok(Flow::Next),
            }
        }
        Op::Coin(c) => {
            let p = f.who(c.flipper);
            match c.mode {
                CoinMode::Single => {
                    g.coin_flip(p, crate::game::CoinCb::Card { card: me, frame: f.frame_at(1) })?;
                }
                CoinMode::Count(n) => {
                    // The sequence callback overwrites the frame's player and loop counters: the
                    // player rides in the resume point.
                    coin_flip_sequence(g, p, n, crate::game::CoinCb::SequenceCard { card: me, frame: f.frame_at(2 + p as u8) })?;
                }
                CoinMode::UntilTails => {
                    coin_flip_sequence(g, p, 0, crate::game::CoinCb::SequenceCard { card: me, frame: f.frame_at(2 + p as u8) })?;
                }
            }
            Ok(Flow::Suspend)
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

pub(crate) fn resume(_g: &mut Game, _me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::Coin(c) => {
            // A finished sequence: the core wrote the results (bit i = flip i heads) and the flip
            // count over the frame's player and loop counters.
            let n = f.iter[0];
            let results = f.p;
            f.p = f.sub - 2;
            f.iter = [0; 4];
            let flips = if n >= 8 { 0xFF } else { (1u8 << n) - 1 };
            f.heads = (results & flips).count_ones() as u8;
            Ok(if c.then.is_empty() { Flow::Next } else { Flow::Enter(2) })
        }
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
        (Op::ForEach(fe), _) => fe.body,
        (Op::Coin(c), 0) => c.heads,
        (Op::Coin(c), 1) => c.tails,
        (Op::Coin(c), _) => c.then,
        _ => &[],
    }
}

pub(crate) fn again(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> bool {
    if let Op::ForEach(fe) = op {
        let pass = f.pass() as usize;
        let list = slots_of(g, me, f, &fe.over);
        return match list.as_slice().get(pass) {
            Some(s) => {
                f.slot = s.p << 4 | s.s;
                true
            }
            None => {
                f.slot = super::super::run::NONE;
                false
            }
        };
    }
    false
}

/// A single flip came up: run the heads or tails list.
pub(crate) fn coin_result(_g: &mut Game, _me: CardId, _f: &mut Frame, op: &Op, heads: bool) -> R<Flow> {
    let Op::Coin(c) = op else { return Ok(Flow::Next) };
    let list = if heads { c.heads } else { c.tails };
    Ok(if list.is_empty() { Flow::Next } else { Flow::Enter(if heads { 0 } else { 1 }) })
}
