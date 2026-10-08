//! The spec interpreter: runs a card's `CardSpec` from the effects it sees.
//!
//! A program (an attack's steps at one rulebook step, a Trainer's effect, an
//! Ability) runs its steps in order. A nested list (the yes branch of a
//! `May`, say) is entered by pushing a path level. An op that asks a question
//! suspends the program: the prompt's continuation is `Cont::Card` with the
//! program's position encoded in the `CardFrame`, and `resume` picks up at the
//! same op. Card registers live in the game's temp lists, which stay alive
//! while any prompt is open.

use super::*;
use crate::cards::CardFrame;
use crate::effects::{AtkBase, EffId, Effect, SlotRef};
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::prefabs::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

/// `CardFrame::stage` of an interpreter frame.
const SPEC_STAGE: u8 = 0xA0;
const MAX_DEPTH: usize = 4;
const NONE: u8 = 0xFF;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Prog {
    /// Position in `spec.attacks`.
    Attack(u8),
    Play,
    /// Position in `spec.powers`.
    Power(u8),
}

/// Which top-level steps a run executes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    BeforeDamage = 0,
    AfterDamage = 1,
    Use = 2,
}

/// A program's position and registers, round-tripped through `CardFrame`.
#[derive(Clone, Copy, Debug)]
struct Frame {
    prog: Prog,
    phase: Phase,
    /// Per level: list selector (bits 7-6: 0 = the program's steps or a yes
    /// branch, 1 = a no branch) and index (bits 5-0).
    path: [u8; MAX_DEPTH],
    depth: u8,
    /// The suspended op's resume point (0 = not suspended).
    sub: u8,
    /// The effect that started the program (the AttackEffect for attacks).
    eff: EffId,
    /// The program's player.
    p: u8,
    /// Card register: a temp list index, or NONE.
    cards: u8,
}

impl Frame {
    fn new(prog: Prog, phase: Phase, eff: EffId, p: usize) -> Frame {
        Frame { prog, phase, path: [0; MAX_DEPTH], depth: 0, sub: 0, eff, p: p as u8, cards: NONE }
    }

    fn encode(&self) -> CardFrame {
        let prog = match self.prog {
            Prog::Attack(i) => i as i32,
            Prog::Play => 0x100,
            Prog::Power(i) => 0x200 | i as i32,
        };
        let mut f = CardFrame::at(SPEC_STAGE | self.phase as u8);
        f.a[0] = prog | (self.depth as i32) << 16 | (self.sub as i32) << 24;
        f.a[1] = i32::from_le_bytes(self.path);
        f.a[2] = self.p as i32;
        f.e[0] = self.eff;
        f.l[0] = self.cards;
        f
    }

    fn decode(f: &CardFrame) -> Option<Frame> {
        if f.stage & 0xF0 != SPEC_STAGE {
            return None;
        }
        let phase = match f.stage & 0x0F {
            0 => Phase::BeforeDamage,
            1 => Phase::AfterDamage,
            _ => Phase::Use,
        };
        let code = f.a[0] & 0xFFFF;
        let prog = match code >> 8 {
            0 => Prog::Attack((code & 0xFF) as u8),
            1 => Prog::Play,
            _ => Prog::Power((code & 0xFF) as u8),
        };
        Some(Frame {
            prog,
            phase,
            path: f.a[1].to_le_bytes(),
            depth: ((f.a[0] >> 16) & 0xFF) as u8,
            sub: ((f.a[0] >> 24) & 0xFF) as u8,
            eff: f.e[0],
            p: f.a[2] as u8,
            cards: f.l[0],
        })
    }

    fn index(&self) -> usize {
        (self.path[self.depth as usize] & 0x3F) as usize
    }

    fn advance(&mut self) {
        self.path[self.depth as usize] += 1;
        self.sub = 0;
    }

    fn enter(&mut self, selector: u8) {
        assert!((self.depth as usize) + 1 < MAX_DEPTH, "spec nesting too deep");
        self.depth += 1;
        self.path[self.depth as usize] = selector << 6;
        self.sub = 0;
    }

    fn opp(&self) -> usize {
        1 - self.p as usize
    }

    fn who(&self, w: Who) -> usize {
        match w {
            Who::Me => self.p as usize,
            Who::Opp => self.opp(),
        }
    }

    /// Continuation resuming this op at resume point `sub`.
    fn cont(&self, me: CardId, sub: u8) -> Cont {
        let mut f = *self;
        f.sub = sub;
        Cont::Card { card: me, frame: f.encode() }
    }
}

enum Flow {
    Next,
    /// Run a nested list (selector 0 = yes/steps, 1 = no).
    Enter(u8),
    /// A prompt is open; `resume` continues.
    Suspend,
}

fn spec_of(g: &Game, me: CardId) -> &'static CardSpec {
    crate::cards::spec_for(g.st.cards[me as usize].def).expect("spec card without a spec")
}

fn program(spec: &'static CardSpec, prog: Prog) -> &'static [Step] {
    match prog {
        Prog::Attack(i) => spec.attacks[i as usize].steps,
        Prog::Play => spec.play.as_ref().map(|p| p.steps).unwrap_or(&[]),
        Prog::Power(i) => spec.powers[i as usize].steps,
    }
}

/// The list the frame's current level walks.
fn list_at(spec: &'static CardSpec, f: &Frame) -> &'static [Step] {
    let mut list = program(spec, f.prog);
    for level in 1..=f.depth as usize {
        let parent = &list[(f.path[level - 1] & 0x3F) as usize];
        let sel = f.path[level] >> 6;
        list = match (&parent.op, sel) {
            (Op::May(m), 0) => m.yes,
            (Op::May(m), _) => m.no,
            (Op::If(i), 0) => i.yes,
            (Op::If(i), _) => i.no,
            _ => &[],
        };
    }
    list
}

// ---------------------------------------------------------------------------
// Entry points

/// `CardImpl::reduce` of every spec card.
pub fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let spec = spec_of(g, me);
    for ps in spec.passives {
        passive(g, me, e, ps)?;
    }
    for (i, a) in spec.attacks.iter().enumerate() {
        if was_attack_used(g, e, a.index, me) {
            if let Some((p, ..)) = attack_data(g, e) {
                run(g, me, Frame::new(Prog::Attack(i as u8), Phase::BeforeDamage, e, p as usize))?;
            }
        } else if after_attack_used(g, e, a.index, me) {
            let atk = real_attack(g, e);
            if let Some((p, ..)) = attack_data(g, atk) {
                run(g, me, Frame::new(Prog::Attack(i as u8), Phase::AfterDamage, atk, p as usize))?;
            }
        }
    }
    if let Some(play) = &spec.play {
        if let Some(p) = trainer_played(g, e, me) {
            if play.kind == PlayKind::Supporter && g.st.players[p].supporter_turn > 0 {
                crate::bail!("SUPPORTER_ALREADY_PLAYED");
            }
            let f = Frame::new(Prog::Play, Phase::Use, e, p);
            if !play.needs.iter().all(|c| cond(g, me, &f, c)) || !implied_ok(g, me, &f, play.steps) {
                crate::bail!("CANNOT_PLAY_THIS_CARD");
            }
            run(g, me, f)?;
        }
    }
    for (i, pw) in spec.powers.iter().enumerate() {
        if was_power_used(g, e, pw.index, me) {
            let p = match *g.e(e) {
                Effect::Power { p, .. } => p as usize,
                _ => continue,
            };
            let f = Frame::new(Prog::Power(i as u8), Phase::Use, e, p);
            if !pw.needs.iter().all(|c| cond(g, me, &f, c)) || !implied_ok(g, me, &f, pw.steps) {
                crate::bail!("CANNOT_USE_POWER");
            }
            run(g, me, f)?;
        }
    }
    Ok(())
}

/// `CardImpl::resume` of every spec card.
pub fn resume(g: &mut Game, me: CardId, cf: CardFrame, results: &[Res]) -> R {
    let Some(mut f) = Frame::decode(&cf) else { return Ok(()) };
    let spec = spec_of(g, me);
    let op = &list_at(spec, &f)[f.index()].op;
    match resume_op(g, me, &mut f, op, results)? {
        Flow::Next => f.advance(),
        Flow::Enter(sel) => f.enter(sel),
        Flow::Suspend => return Ok(()),
    }
    run(g, me, f)
}

fn run(g: &mut Game, me: CardId, mut f: Frame) -> R {
    let spec = spec_of(g, me);
    loop {
        let list = list_at(spec, &f);
        let i = f.index();
        if i >= list.len() {
            if f.depth == 0 {
                return Ok(());
            }
            f.depth -= 1;
            f.advance();
            continue;
        }
        let step = &list[i];
        if f.depth == 0 && !runs_in(step.at, f.phase) {
            f.advance();
            continue;
        }
        match exec(g, me, &mut f, &step.op)? {
            Flow::Next => f.advance(),
            Flow::Enter(sel) => f.enter(sel),
            Flow::Suspend => return Ok(()),
        }
    }
}

fn runs_in(at: RuleStep, phase: Phase) -> bool {
    matches!(
        (at, phase),
        (RuleStep::BeforeDamage, Phase::BeforeDamage) | (RuleStep::AfterDamage, Phase::AfterDamage) | (RuleStep::Use, Phase::Use)
    )
}

// ---------------------------------------------------------------------------
// Ops

fn exec(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
        Op::Draw(d) => {
            let p = f.who(d.who);
            let n = match &d.amount {
                DrawAmount::Count(n) => num(g, me, f, n),
                DrawAmount::UntilHandSize(n) => num(g, me, f, n) - g.st.players[p].hand.len() as i32,
            };
            if n > 0 {
                draw_cards(g, p, n as usize)?;
            }
            Ok(Flow::Next)
        }
        Op::May(m) => {
            if !cond(g, me, f, &m.when) {
                return Ok(Flow::Next);
            }
            confirmation_prompt(g, f.who(m.asker), m.msg, f.cont(me, 1));
            Ok(Flow::Suspend)
        }
        Op::If(i) => {
            if cond(g, me, f, &i.cond) {
                Ok(if i.yes.is_empty() { Flow::Next } else { Flow::Enter(0) })
            } else {
                Ok(if i.no.is_empty() { Flow::Next } else { Flow::Enter(1) })
            }
        }
        Op::Search(s) => search(g, me, f, s),
        Op::Shuffle(s) => {
            let p = f.who(s.zone.0);
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, f.cont(me, 1));
            Ok(Flow::Suspend)
        }
        Op::DiscardEnergy(d) => {
            let slot = slot_of(g, me, f, d.target);
            let Some(slot) = slot else { return Ok(Flow::Next) };
            let (p, opp, attack, source) = match attack_data(g, f.eff) {
                Some(x) => x,
                None => return Ok(Flow::Next),
            };
            match d.selection {
                EnergySelection::AllProvided => {
                    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: slot.p, source: slot, energy_map: SVec::new() })?;
                    let mut cards: SVec<CardId, 64> = SVec::new();
                    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                        for m in energy_map.iter() {
                            cards.push(m.card);
                        }
                    }
                    let b = AtkBase { attack_effect: f.eff, player: p, opponent: opp, attack, source, target: slot };
                    g.run_fx(Effect::DiscardCards { b, cards })?;
                }
            }
            Ok(Flow::Next)
        }
        Op::HandShuffleDraw(h) => {
            let p = f.who(h.who);
            let n = num(g, me, f, &h.draw).max(0) as u8;
            let mut after = *f;
            after.sub = 1;
            shuffle_hand_into_deck_then_draw_ex(g, p, me, NO_CARD, n, Some((me, after.encode())))?;
            Ok(Flow::Suspend)
        }
        Op::RemoveFromPlay(r) => {
            if let Some(slot) = slot_of(g, me, f, r.slot) {
                let dst = zone_ref(f, r.destination);
                move_pokemon_off_board(g, slot, dst, me)?;
            }
            Ok(Flow::Next)
        }
    }
}

fn resume_op(g: &mut Game, _me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::May(m) => {
            if first.as_bool() {
                Ok(if m.yes.is_empty() { Flow::Next } else { Flow::Enter(0) })
            } else {
                Ok(if m.no.is_empty() { Flow::Next } else { Flow::Enter(1) })
            }
        }
        Op::Search(s) => {
            let p = f.who(s.pick.chooser);
            let chosen: Vec<CardId> = first.cards().to_vec();
            match s.destination {
                SearchDestination::Bench => {
                    let open = empty_bench_slots(g, p);
                    for (c, slot) in chosen.iter().zip(open.iter()) {
                        g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, *slot) })?;
                    }
                }
            }
            Ok(Flow::Next)
        }
        Op::Shuffle(s) => {
            if let Res::Order(o) = first {
                let p = f.who(s.zone.0);
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(Flow::Next)
        }
        // Resumed after the prefab's draw.
        Op::HandShuffleDraw(_) => Ok(Flow::Next),
        _ => Ok(Flow::Next),
    }
}

fn search(g: &mut Game, me: CardId, f: &mut Frame, s: &SearchSpec) -> R<Flow> {
    let p = f.who(s.pick.chooser);
    let from = zone_ref(f, s.pick.from);
    let open = match s.destination {
        SearchDestination::Bench => empty_bench_slots(g, p).len() as i32,
    };
    if g.lst(from).is_empty() || open == 0 {
        // Can't be carried out: nothing is searched (an attack still resolves;
        // a Trainer or Ability is not playable, see `implied_ok`).
        return Ok(Flow::Next);
    }
    let max = num(g, me, f, &s.pick.bounds.max).min(open).max(0) as u8;
    let min = num(g, me, f, &s.pick.bounds.min).min(max as i32).max(0) as u8;
    let mut opts = ChooseCardsOpts::new(min, max, false);
    for (i, c) in g.lst(from).iter().enumerate() {
        if !pred(g, *c, &s.pick.predicate) {
            opts.blocked.push(i as u8);
        }
    }
    choose_cards(g, p, s.msg, from, Filter::none(), opts, f.cont(me, 1));
    Ok(Flow::Suspend)
}

/// Preconditions the ops themselves imply for a Trainer or an Ability (an
/// attack can be used even when its effects can't be carried out).
fn implied_ok(g: &Game, me: CardId, f: &Frame, steps: &[Step]) -> bool {
    let _ = me;
    steps.iter().all(|s| match &s.op {
        Op::Search(x) => {
            let p = f.who(x.pick.chooser);
            let from = zone_ref(f, x.pick.from);
            let room = match x.destination {
                SearchDestination::Bench => !empty_bench_slots(g, p).is_empty(),
            };
            !g.lst(from).is_empty() && room
        }
        _ => true,
    })
}

// ---------------------------------------------------------------------------
// Passives

fn passive(g: &mut Game, me: CardId, e: EffId, ps: &Passive) -> R {
    match ps.modifier {
        Modifier::HpBonus(n) => {
            let (p, target, card) = match *g.e(e) {
                Effect::CheckHp { p, target, card } => (p as usize, target, card),
                _ => return Ok(()),
            };
            match ps.origin {
                RuleSource::Tool => {
                    if !g.st.slot(target.p as usize, target.s).tools.contains(me) || is_tool_blocked(g, p, me) {
                        return Ok(());
                    }
                }
            }
            // HP is only raised for a Pokémon actually being checked.
            if card.is_some() {
                g.st.players[target.p as usize].slots[target.s as usize].hp_bonus += n;
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Values

fn zone_ref(f: &Frame, z: ZoneRef) -> ListRef {
    let p = f.who(z.0) as u8;
    match z.1 {
        Zone::Deck => ListRef::Deck(p),
        Zone::Hand => ListRef::Hand(p),
        Zone::Discard => ListRef::Discard(p),
    }
}

fn slot_of(g: &Game, me: CardId, f: &Frame, s: SlotExpr) -> Option<SlotRef> {
    match s {
        SlotExpr::Active(w) => {
            let p = f.who(w);
            Some(SlotRef::new(p, g.st.players[p].active))
        }
        SlotExpr::This => {
            for p in 0..2 {
                let pl = &g.st.players[p];
                let mut slots: Vec<SlotId> = vec![pl.active];
                slots.extend(pl.bench.iter().copied());
                for s in slots {
                    let sl = &pl.slots[s as usize];
                    if sl.cards.contains(me) || sl.tools.contains(me) {
                        return Some(SlotRef::new(p, s));
                    }
                }
            }
            None
        }
    }
}

fn num(g: &Game, me: CardId, f: &Frame, n: &Num) -> i32 {
    match n {
        Num::Lit(v) => *v,
        Num::ZoneSize(z) => g.lst(zone_ref(f, *z)).len() as i32,
        Num::OpenBench(w) => empty_bench_slots(g, f.who(*w)).len() as i32,
        Num::PrizesLeft(w) => g.st.players[f.who(*w)].prize_left() as i32,
        Num::Min(a, b) => num(g, me, f, a).min(num(g, me, f, b)),
        Num::If(c, a, b) => {
            if cond(g, me, f, c) {
                num(g, me, f, a)
            } else {
                num(g, me, f, b)
            }
        }
    }
}

fn cond(g: &Game, me: CardId, f: &Frame, c: &Cond) -> bool {
    match c {
        Cond::True => true,
        Cond::Not(c) => !cond(g, me, f, c),
        Cond::All(cs) => cs.iter().all(|c| cond(g, me, f, c)),
        Cond::Cmp(a, op, b) => {
            let (a, b) = (num(g, me, f, a), num(g, me, f, b));
            match op {
                CmpOp::Lt => a < b,
                CmpOp::Le => a <= b,
                CmpOp::Eq => a == b,
                CmpOp::Ge => a >= b,
                CmpOp::Gt => a > b,
            }
        }
        Cond::Nonempty(z, p) => g.lst(zone_ref(f, *z)).iter().any(|c| pred(g, *c, p)),
        Cond::BenchSpace(w) => !empty_bench_slots(g, f.who(*w)).is_empty(),
    }
}

fn pred(g: &Game, c: CardId, p: &Pred) -> bool {
    let d = g.st.cdef(c);
    match p {
        Pred::Any => true,
        Pred::All(ps) => ps.iter().all(|p| pred(g, c, p)),
        Pred::Pokemon => d.is_pokemon(),
        Pred::Basic => d.is_pokemon() && d.stage == Stage::Basic as u8,
        Pred::HpAtMost(n) => d.is_pokemon() && d.hp <= *n,
    }
}
