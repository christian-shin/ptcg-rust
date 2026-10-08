//! Control flow: May, If, Coin, Choose, loops, Fail, attack copying, ending
//! the turn or the game, Custom (vocabulary v1 "Operations", control flow).
//!
//! A nested list is entered with `Flow::Enter(sel)`; `child` names the list
//! for each selector, and `again` decides whether a finished list runs again
//! (loops count passes in `Frame::iter`).

use super::super::run::{Flow, Frame, CHOICE_NO, CHOICE_NONE, CHOICE_YES};
use super::super::*;
use crate::game::{CoinCb, Game, R};
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
/// How many coins a `Coin` flips.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoinMode {
    /// One flip: `heads` or `tails` runs.
    One,
    /// This many flips, then `then` runs (`Num::Heads` counts the heads).
    Fixed(u8),
    /// Flip until tails, then `then` runs.
    UntilTails,
}
pub struct CoinSpec {
    pub mode: CoinMode,
    pub heads: &'static [Step],
    pub tails: &'static [Step],
    pub then: &'static [Step],
}
impl CoinSpec {
    pub const DEFAULT: CoinSpec = CoinSpec { mode: CoinMode::One, heads: &[], tails: &[], then: &[] };
}
pub struct ChooseSpec {}
pub struct ForEachSpec {}
pub struct RepeatSpec {}
pub struct ParallelSpec {}
/// The card can't be used (an attack can't be declared, a Trainer can't be played) when `when` holds.
pub struct FailSpec {
    pub when: Cond,
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
/// Choose 1 of the attacks of the Pokémon cards in a register (still in the discard pile) and use it
/// as this attack, as a copy session: no cancel, `retries` attempts when a chosen attack can't be used.
pub struct CopyFromRegSpec {
    pub reg: u8,
    pub retries: u8,
}

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
        Op::Coin(c) => {
            let p = f.p as usize;
            match c.mode {
                CoinMode::One => {
                    g.coin_flip(p, CoinCb::Card { card: me, frame: f.frame_at(1) })?;
                }
                CoinMode::Fixed(_) | CoinMode::UntilTails => {
                    // The sequence's result overwrites the frame's player and loop counters:
                    // keep them on the side.
                    g.spec_choices.retain(|x| !(x.card == me && x.key == COIN_STASH));
                    let mut ch = SpecChoice { card: me, key: COIN_STASH, answer: f.p, items: [0; 16], len: 4 };
                    ch.items[..4].copy_from_slice(&f.iter);
                    g.spec_choices.push(ch);
                    let n = if let CoinMode::Fixed(n) = c.mode { n } else { 0 };
                    coin_flip_sequence(g, p, n, CoinCb::SequenceCard { card: me, frame: f.frame_at(1) })?;
                }
            }
            Ok(Flow::Suspend)
        }
        Op::CopyFromReg(c) => {
            let p = f.p as usize;
            // The Pokémon cards of the register that are still in the discard pile.
            let cards: Vec<CardId> = reg_list(g, f, c.reg).iter().copied().filter(|x| g.lst(crate::state::ListRef::Discard(p as u8)).contains(x)).collect();
            if cards.is_empty() {
                return Ok(Flow::Next);
            }
            // Nothing is chosen when every attack of those Pokémon is locked for the Active.
            let a = g.st.players[p].active;
            let locked = g.st.slot(p, a).cannot_use_attacks_next_turn;
            let any_free = cards.iter().any(|x| g.st.cdef(*x).attacks.iter().any(|at| !locked.iter().any(|n| *n == at.tl_name)));
            if !any_free {
                return Ok(Flow::Next);
            }
            crate::copy_attack::copy_attack_from_pokemon_list_retries(g, f.eff, &cards, false, c.retries)?;
            Ok(Flow::Next)
        }
        Op::Fail(x) => {
            if cond_m(g, me, f, &x.when)? {
                crate::bail!(x.error);
            }
            Ok(Flow::Next)
        }
        _ => unimplemented!("spec op not implemented yet (ops/flow.rs)"),
    }
}

/// Key of the side copy of a frame kept across a coin sequence.
const COIN_STASH: u64 = u64::MAX;

/// A single coin flip of `op` came up `heads`.
pub(crate) fn coin(_g: &mut Game, _me: CardId, _f: &mut Frame, op: &Op, heads: bool) -> R<Flow> {
    match op {
        Op::Coin(c) => Ok(if heads {
            if c.heads.is_empty() {
                Flow::Next
            } else {
                Flow::Enter(0)
            }
        } else if c.tails.is_empty() {
            Flow::Next
        } else {
            Flow::Enter(1)
        }),
        _ => Ok(Flow::Next),
    }
}

pub(crate) fn resume(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::Coin(c) => {
            // A flip sequence is over: its results replaced the frame's player (low byte) and counters.
            let heads = f.p.count_ones() as u8;
            if let Some(i) = g.spec_choices.iter().position(|x| x.card == me && x.key == COIN_STASH) {
                let st = g.spec_choices.remove_at(i);
                f.p = st.answer;
                f.iter.copy_from_slice(&st.items[..4]);
            }
            f.heads = heads;
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
        (Op::Coin(c), 0) => c.heads,
        (Op::Coin(c), 1) => c.tails,
        (Op::Coin(c), _) => c.then,
        _ => &[],
    }
}

pub(crate) fn again(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op) -> bool {
    false
}
