//! Control flow: May, If, Coin, Choose, loops, Fail, attack copying, ending
//! the turn or the game, Custom (vocabulary v1 "Operations", control flow).
//!
//! A nested list is entered with `Flow::Enter(sel)`; `child` names the list
//! for each selector, and `again` decides whether a finished list runs again
//! (loops count passes in `Frame::iter`).

use super::super::run::{Flow, Frame, CHOICE_NO, CHOICE_NONE, CHOICE_YES};
use super::super::*;
use crate::effects::Effect;
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
/// Flip coins (the player the program runs for). One flip runs `heads` or `tails`; counted
/// flips (`Count`, `UntilTails`) apply `per_heads` to the attack's damage and then run `heads`
/// once (it can read the number of heads with `Num::CoinHeads`).
pub struct CoinSpec {
    pub flips: Flips,
    pub per_heads: PerHeads,
    pub heads: &'static [Step],
    pub tails: &'static [Step],
}
impl CoinSpec {
    pub const DEFAULT: CoinSpec = CoinSpec { flips: Flips::One, per_heads: PerHeads::Nothing, heads: &[], tails: &[] };
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Flips {
    One,
    /// A fixed number of flips (a MULTIPLE_COIN_FLIPS prompt).
    Count(u8),
    /// Flip until tails; the number of heads is what counts.
    UntilTails,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PerHeads {
    Nothing,
    /// "N damage for each heads": the attack's damage is N times the heads.
    DamageIs(i32),
    /// "N more damage for each heads".
    DamageAdd(i32),
}
pub struct ChooseSpec {}
/// Run `body` once for each Pokémon of the selection, in order, with that Pokémon as the picked slot
/// (`SlotExpr::Picked`).
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
    pub scope: CopyScope,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CopyScope {
    /// The Active Pokémon's attacks.
    Active,
    // --- S3 agent 3 appends ---
    /// The attacks of the Benched Pokémon matching `predicate` (chosen among them; N's Zoroark ex).
    Bench,
}
pub struct EndTurnSpec {}
/// The game ends and `winner` wins.
pub struct EndGameSpec {
    pub winner: Who,
}
/// A card's own steps, for the cards whose text no vocabulary item expresses (Mr. Mime, Backtrack
/// Badge): `exec` runs the step, `resume` continues after the prompt it opened (`Frame::cont`).
pub struct CustomSpec {
    pub exec: fn(&mut Game, CardId, &mut Frame) -> R<Flow>,
    pub resume: fn(&mut Game, CardId, &mut Frame, &[Res]) -> R<Flow>,
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
            if c.scope == CopyScope::Bench {
                let cards: Vec<CardId> = g.st.players[p].bench.iter().filter_map(|b| g.st.slot_pokemon(p, *b)).filter(|c0| pred(g, *c0, &c.predicate)).collect();
                if cards.is_empty() {
                    return Ok(Flow::Next);
                }
                crate::copy_attack::copy_attack_from_pokemon_list_retries(g, f.eff, &cards, false, c.retries)?;
                return Ok(Flow::Next);
            }
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
        Op::Custom(c) => (c.exec)(g, me, f),
        Op::ForEach(fe) => {
            let v = slots_of(g, me, f, &fe.over);
            let Some(first) = v.as_slice().first() else { return Ok(Flow::Next) };
            f.slot = first.p << 4 | first.s;
            Ok(if fe.body.is_empty() { Flow::Next } else { Flow::Enter(0) })
        }
        Op::Coin(c) => {
            let p = f.p as usize;
            let frame = f.frame_at(if c.flips == Flips::One { 1 } else { super::super::run::COIN_SEQUENCE });
            match c.flips {
                Flips::One => {
                    g.coin_flip(p, CoinCb::Card { card: me, frame })?;
                }
                Flips::Count(n) => coin_flip_sequence(g, p, n, CoinCb::SequenceCard { card: me, frame })?,
                Flips::UntilTails => coin_flip_sequence(g, p, 0, CoinCb::SequenceCard { card: me, frame })?,
            }
            Ok(Flow::Suspend)
        }
        _ => unimplemented!("spec op not implemented yet (ops/flow.rs)"),
    }
}

/// The result of a coin op: `bits` (bit i = flip i was heads) of `n` flips.
pub(crate) fn resume_coin(g: &mut Game, _me: CardId, f: &mut Frame, op: &Op, bits: u32, n: u8) -> R<Flow> {
    let Op::Coin(c) = op else { return Ok(Flow::Next) };
    let heads = (bits & ((1u64 << n) - 1) as u32).count_ones() as i32;
    f.set_heads(heads as u8);
    if c.per_heads != PerHeads::Nothing {
        if let Effect::Attack { damage, .. } = g.e_mut(f.eff) {
            match c.per_heads {
                PerHeads::Nothing => {}
                PerHeads::DamageIs(k) => *damage = k * heads,
                PerHeads::DamageAdd(k) => *damage += k * heads,
            }
        }
    }
    Ok(match c.flips {
        Flips::One if heads > 0 => if c.heads.is_empty() { Flow::Next } else { Flow::Enter(0) },
        Flips::One => if c.tails.is_empty() { Flow::Next } else { Flow::Enter(1) },
        _ => if c.heads.is_empty() { Flow::Next } else { Flow::Enter(0) },
    })
}

pub(crate) fn resume(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::Custom(c) => (c.resume)(g, me, f, results),
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
        (Op::Coin(c), _) => c.tails,
        _ => &[],
    }
}

pub(crate) fn again(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> bool {
    if let Op::ForEach(fe) = op {
        let k = f.pass() as usize;
        let v = slots_of(g, me, f, &fe.over);
        if let Some(s) = v.as_slice().get(k) {
            f.slot = s.p << 4 | s.s;
            return true;
        }
    }
    false
}
