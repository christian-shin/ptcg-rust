//! Control flow: May, If, Coin, Choose, loops, Fail, attack copying, ending
//! the turn or the game, Custom (vocabulary v1 "Operations", control flow).
//!
//! A nested list is entered with `Flow::Enter(sel)`; `child` names the list
//! for each selector, and `again` decides whether a finished list runs again
//! (loops count passes in `Frame::iter`).

use super::super::run::{Flow, Frame, CHOICE_NO, CHOICE_NONE, CHOICE_YES, COIN_SEQUENCE};
use super::super::*;
use crate::effects::Effect;
use crate::game::{CoinCb, Game, R};
use crate::list::{CardId, SVec};
use crate::prefabs::*;
use crate::prompts::{PromptKind, Res, SelectValues};
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
/// "Flip a coin(s)": the flipper flips (when `before` holds, otherwise nothing is flipped), then the branch
/// for the result runs. One flip runs `heads` or
/// `tails`. A sequence (`Count`, `UntilTails`) applies `per_heads` to the attack's damage, then runs
/// `then` once; `Num::Heads` reads the number of heads (also after a single flip).
pub struct CoinSpec {
    pub flipper: Who,
    pub flips: Flips,
    pub before: Cond,
    pub per_heads: PerHeads,
    pub heads: &'static [Step],
    pub tails: &'static [Step],
    /// After the flips of a sequence.
    pub then: &'static [Step],
}
impl CoinSpec {
    pub const DEFAULT: CoinSpec = CoinSpec { flipper: Who::Me, flips: Flips::One, before: Cond::True, per_heads: PerHeads::Nothing, heads: &[], tails: &[], then: &[] };
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
/// One option of a `Choose`: shown when `avail` holds; `body` runs when it is chosen.
pub struct ChoiceBranch {
    pub label: &'static str,
    pub avail: Cond,
    pub body: &'static [Step],
}

/// The chooser picks one of the available options (a Select prompt; nothing happens when none is
/// available). At most 7 options.
pub struct ChooseSpec {
    pub chooser: Who,
    pub msg: &'static str,
    pub options: &'static [ChoiceBranch],
}
/// Run `body` once for each selected Pokémon, in order; inside the body the Pokémon is the
/// picked slot (`SlotExpr::Picked`). The list is read when the loop starts and again at each
/// pass by position.
pub struct ForEachSpec {
    pub over: SlotSel,
    pub body: &'static [Step],
}
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
/// The player's turn ends (an EndTurnEffect).
pub struct EndTurnSpec {
    pub who: Who,
}
/// The game ends and `winner` wins.
pub struct EndGameSpec {
    pub winner: Who,
}
/// Code kept in a card file for a rule no op expresses (Mr. Mime, Backtrack Badge): the card's own
/// exec and resume functions, run as one op of a program (`resume` continues after the prompt it
/// opened, `Frame::cont`). The state a suspended program needs travels in the frame.
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
        Op::Custom(c) => (c.exec)(g, me, f),
        Op::Choose(c) => {
            let mut mask = 0u8;
            let mut labels: SVec<&'static str, 8> = SVec::new();
            for (i, b) in c.options.iter().enumerate() {
                if cond_m(g, me, f, &b.avail)? {
                    mask |= 1 << i;
                    labels.push(b.label);
                }
            }
            if labels.is_empty() {
                return Ok(Flow::Next);
            }
            let id = g.player_id(f.who(c.chooser));
            g.prompt(id, c.msg, PromptKind::Select { values: SelectValues::Dyn(labels), allow_cancel: false, default_value: 0 }, f.cont(me, 0x80 | mask));
            Ok(Flow::Suspend)
        }
        Op::EndTurn(t) => {
            g.run_fx(crate::effects::Effect::EndTurn { p: f.who(t.who) as u8 })?;
            Ok(Flow::Next)
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
            if !cond_m(g, me, f, &c.before)? {
                return Ok(Flow::Next);
            }
            let p = f.who(c.flipper);
            match c.flips {
                Flips::One => {
                    g.coin_flip(p, CoinCb::Card { card: me, frame: f.frame_at(1) })?;
                }
                // The sequence's callback leaves the frame as it is and passes the results.
                Flips::Count(n) => coin_flip_sequence(g, p, n, CoinCb::SequenceCard { card: me, frame: f.frame_at(COIN_SEQUENCE) })?,
                Flips::UntilTails => coin_flip_sequence(g, p, 0, CoinCb::SequenceCard { card: me, frame: f.frame_at(COIN_SEQUENCE) })?,
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
        _ => unimplemented!("spec op not implemented yet (ops/flow.rs)"),
    }
}

/// A finished coin sequence (`results` are the bits, bit i = flip i was heads, and the flip count):
/// `per_heads` changes the damage, then `then` runs.
pub(crate) fn resume_coin(g: &mut Game, _me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let Op::Coin(c) = op else { return Ok(Flow::Next) };
    let bits = results.first().map_or(0, |r| r.as_int()) as u32;
    let n = results.get(1).map_or(0, |r| r.as_int()) as u32;
    let heads = (bits & (((1u64 << n) - 1) as u32)).count_ones() as u8;
    flips_done(g, f, c, heads);
    Ok(if c.then.is_empty() { Flow::Next } else { Flow::Enter(2) })
}

/// The flips are done with `heads` of them heads.
fn flips_done(g: &mut Game, f: &mut Frame, c: &CoinSpec, heads: u8) {
    f.heads = heads;
    if c.per_heads != PerHeads::Nothing {
        if let Effect::Attack { damage, .. } = g.e_mut(f.eff) {
            match c.per_heads {
                PerHeads::Nothing => {}
                PerHeads::DamageIs(k) => *damage = k * heads as i32,
                PerHeads::DamageAdd(k) => *damage += k * heads as i32,
            }
        }
    }
}

pub(crate) fn resume(g: &mut Game, me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::Custom(c) => (c.resume)(g, me, f, results),
        Op::Choose(c) => {
            let idx = first.as_int();
            let mask = f.sub & 0x7F;
            let branch = (0..c.options.len()).filter(|i| mask & (1 << i) != 0).nth(idx.max(0) as usize);
            match branch {
                Some(i) if idx >= 0 => Ok(if c.options[i].body.is_empty() { Flow::Next } else { Flow::Enter(i as u8) }),
                _ => crate::bail!("TypeError: Cannot read properties of undefined (reading 'action')"),
            }
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
        (Op::Choose(c), sel) => c.options.get(sel as usize).map(|o| o.body).unwrap_or(&[]),
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
        let list = slots_m(g, me, f, &fe.over).unwrap_or_default();
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

/// A single flip came up: `per_heads` changes the damage, then the heads or tails list runs.
pub(crate) fn coin_result(g: &mut Game, _me: CardId, f: &mut Frame, op: &Op, heads: bool) -> R<Flow> {
    let Op::Coin(c) = op else { return Ok(Flow::Next) };
    flips_done(g, f, c, heads as u8);
    let list = if heads { c.heads } else { c.tails };
    Ok(if list.is_empty() { Flow::Next } else { Flow::Enter(if heads { 0 } else { 1 }) })
}
