//! The spec interpreter: runs a card's `CardSpec` from the effects it sees.
//!
//! A program (an attack's steps at one rulebook step, a Trainer's effect, an
//! Ability, a trigger) runs its steps in order. A nested list (the yes branch
//! of a `May`, a loop body) is entered by pushing a path level. An op that
//! asks a question suspends the program: the prompt's continuation is
//! `Cont::Card` with the program's position encoded in the `CardFrame`, and
//! `resume` picks up at the same op. Card registers live in the game's temp
//! lists, which stay alive while any prompt is open; attack choices made at
//! step D live in `Game::spec_choices` until the attack's effects are done.

use super::*;
use crate::cards::CardFrame;
use crate::effects::{EffId, Effect};
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::prefabs::*;
use crate::prompts::*;

/// `CardFrame::stage` of an interpreter frame (low 4 bits: the phase).
const SPEC_STAGE: u8 = 0xA0;
const MAX_DEPTH: usize = 4;
/// Path element: list selector in bits 7-5, step index in bits 4-0.
const SEL_SHIFT: u8 = 5;
const INDEX_MASK: u8 = 0x1F;
pub(crate) const NONE: u8 = 0xFF;
/// Resume point of a finished coin sequence (`CoinCb::SequenceCard`).
pub(crate) const COIN_SEQUENCE: u8 = 0x7E;

pub(crate) const CHOICE_NO: u8 = 0;
pub(crate) const CHOICE_YES: u8 = 1;
/// Nothing was asked at step D (no possible effect then).
pub(crate) const CHOICE_NONE: u8 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Prog {
    /// Position in `spec.attacks`.
    Attack(u8),
    Play,
    /// Position in `spec.powers`.
    Power(u8),
    /// Position in `spec.triggers`.
    Trigger(u8),
    /// `spec.use_stadium`.
    UseStadium,
}

/// Which top-level steps a run executes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Phase {
    BeforeDamage = 0,
    AfterDamage = 1,
    Use = 2,
    /// Step D of an attack (FINAL attack-choice rule, user 2026-10-07): the
    /// choices of the after-damage steps are made now, before the damage, and
    /// recorded in `Game::spec_choices`; the after-damage run carries them out.
    /// Searches and choices nested under conditions, coins or loops are still
    /// made after the damage, when their options exist.
    Choices = 3,
}

/// A program's position and registers, round-tripped through `CardFrame`.
///
/// Layout (`CardFrame { a: [i32; 4], e: [u8; 2], l: [u8; 2] }`):
/// - `a[0]`: bits 0-15 program code, 16-19 depth, 20-23 heads, 24-31 resume point (`sub`);
/// - `a[1]`: the path, one byte per level;
/// - `a[2]`: bit 0 player `p`, 1-4 prize pile (15: none), 5 `via_attack`,
///   8-15 `attached_to`, 16-31 `last`;
/// - `a[3]`: the loop counters, one byte per level;
/// - `e[0]`: the effect; `e[1]`: the picked slot (`p << 4 | slot`);
/// - `l`: the two card registers.
/// A finished coin sequence passes its results as resume values, so nothing here is overwritten.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub(crate) prog: Prog,
    pub(crate) phase: Phase,
    pub(crate) path: [u8; MAX_DEPTH],
    pub(crate) depth: u8,
    /// Per level: passes of the loop that owns the next level.
    pub(crate) iter: [u8; MAX_DEPTH],
    /// The suspended op's resume point (0 = not suspended).
    pub(crate) sub: u8,
    /// The effect that started the program (the AttackEffect for attacks).
    pub(crate) eff: EffId,
    /// The program's player.
    pub(crate) p: u8,
    /// Card registers: temp list indices, or NONE.
    pub(crate) cards: [u8; 2],
    /// Heads of the last finished coin sequence or single flip (`Num::Heads`).
    pub(crate) heads: u8,
    /// The Pokémon picked by the last `PickSlot`, or the one an event is about (a trigger's
    /// attacking Pokémon), as `p << 4 | slot`; NONE when there is none.
    pub(crate) slot: u8,
    /// The Prize card pile picked by the last `PickPrize`, or NONE.
    pub(crate) prize: u8,
    /// The Pokémon the last `Attach` attached cards to (`p << 4 | slot`), or NONE.
    pub(crate) attached_to: u8,
    /// Cards the last discard of the program moved (`Num::Last`, 16 bits).
    pub(crate) last: i32,
    /// The Trainer is used as the effect of an attack (Look-Alike Show); fixed when it starts.
    pub(crate) via_attack: bool,
}

impl Frame {
    pub(crate) fn new(prog: Prog, phase: Phase, eff: EffId, p: usize) -> Frame {
        Frame { prog, phase, path: [0; MAX_DEPTH], depth: 0, iter: [0; MAX_DEPTH], sub: 0, eff, p: p as u8, cards: [NONE; 2], heads: 0, slot: NONE, prize: NONE, attached_to: NONE, last: 0, via_attack: false }
    }

    fn prog_code(&self) -> u32 {
        match self.prog {
            Prog::Attack(i) => i as u32,
            Prog::Play => 0x100,
            Prog::Power(i) => 0x200 | i as u32,
            Prog::Trigger(i) => 0x300 | i as u32,
            Prog::UseStadium => 0x400,
        }
    }

    fn encode(&self) -> CardFrame {
        let mut f = CardFrame::at(SPEC_STAGE | self.phase as u8);
        f.a[0] = (self.prog_code() | ((self.depth as u32) | (self.heads as u32) << 4) << 16 | (self.sub as u32) << 24) as i32;
        f.a[1] = i32::from_le_bytes(self.path);
        f.a[2] = (self.p as i32 & 1)
            | ((self.prize & 15) as i32) << 1
            | (self.via_attack as i32) << 5
            | (self.attached_to as i32) << 8
            | ((self.last & 0xFFFF) << 16);
        f.a[3] = i32::from_le_bytes(self.iter);
        f.e[0] = self.eff;
        f.e[1] = self.slot;
        f.l = self.cards;
        f
    }

    fn decode(f: &CardFrame) -> Option<Frame> {
        if f.stage & 0xF0 != SPEC_STAGE {
            return None;
        }
        let phase = match f.stage & 0x0F {
            0 => Phase::BeforeDamage,
            1 => Phase::AfterDamage,
            3 => Phase::Choices,
            _ => Phase::Use,
        };
        let a0 = f.a[0] as u32;
        let code = a0 & 0xFFFF;
        let prog = match code >> 8 {
            0 => Prog::Attack((code & 0xFF) as u8),
            1 => Prog::Play,
            2 => Prog::Power((code & 0xFF) as u8),
            3 => Prog::Trigger((code & 0xFF) as u8),
            _ => Prog::UseStadium,
        };
        Some(Frame {
            prog,
            phase,
            path: f.a[1].to_le_bytes(),
            depth: ((a0 >> 16) & 0x0F) as u8,
            heads: ((a0 >> 20) & 0x0F) as u8,
            iter: f.a[3].to_le_bytes(),
            sub: ((a0 >> 24) & 0xFF) as u8,
            eff: f.e[0],
            slot: f.e[1],
            p: (f.a[2] & 1) as u8,
            prize: match (f.a[2] >> 1) & 15 {
                15 => NONE,
                n => n as u8,
            },
            via_attack: (f.a[2] >> 5) & 1 != 0,
            attached_to: ((f.a[2] >> 8) & 0xFF) as u8,
            last: (f.a[2] >> 16) as i16 as i32,
            cards: f.l,
        })
    }

    pub(crate) fn index(&self) -> usize {
        (self.path[self.depth as usize] & INDEX_MASK) as usize
    }

    fn advance(&mut self) {
        self.path[self.depth as usize] += 1;
        self.sub = 0;
    }

    fn enter(&mut self, sel: u8) {
        assert!((self.depth as usize) + 1 < MAX_DEPTH, "spec nesting too deep");
        self.iter[self.depth as usize] = 0;
        self.depth += 1;
        self.path[self.depth as usize] = sel << SEL_SHIFT;
        self.sub = 0;
    }

    /// Passes of the loop whose body is the current level.
    #[allow(dead_code)]
    pub(crate) fn pass(&self) -> u8 {
        if self.depth == 0 {
            0
        } else {
            self.iter[self.depth as usize - 1]
        }
    }

    /// The frame that resumes this op at resume point `sub`.
    pub(crate) fn frame_at(&self, sub: u8) -> CardFrame {
        let mut f = *self;
        f.sub = sub;
        f.encode()
    }

    /// Continuation resuming this op at resume point `sub`.
    pub(crate) fn cont(&self, me: CardId, sub: u8) -> Cont {
        Cont::Card { card: me, frame: self.frame_at(sub) }
    }

    /// Identifies the current step among this card's programs.
    fn key(&self) -> u64 {
        (self.prog_code() as u64) << 40 | (self.depth as u64) << 32 | u32::from_le_bytes(self.path) as u64
    }

    /// Record the step-D answer of the current step.
    pub(crate) fn record(&self, g: &mut Game, me: CardId, answer: u8) {
        self.record_items(g, me, answer, &[]);
    }

    /// Record the step-D answer of the current step with the chosen items
    /// (at most `SPEC_CHOICE_ITEMS` bytes, encoded by the op).
    pub(crate) fn record_items(&self, g: &mut Game, me: CardId, answer: u8, items: &[u8]) {
        let key = self.key();
        g.spec_choices.retain(|c| !(c.card == me && c.key == key));
        assert!(items.len() <= SPEC_CHOICE_ITEMS, "step-D answer with {} items", items.len());
        let mut c = SpecChoice { card: me, key, answer, items: [0; SPEC_CHOICE_ITEMS], len: items.len() as u8 };
        c.items[..items.len()].copy_from_slice(items);
        g.spec_choices.push(c);
    }

    /// The step-D answer of the current step, when carrying out an attack's
    /// effects after the damage.
    pub(crate) fn recorded(&self, g: &Game, me: CardId) -> Option<u8> {
        self.recorded_choice(g, me).map(|c| c.answer)
    }

    /// The step-D choice of the current step (answer and items), when
    /// carrying out an attack's effects after the damage.
    pub(crate) fn recorded_choice(&self, g: &Game, me: CardId) -> Option<SpecChoice> {
        if self.phase != Phase::AfterDamage {
            return None;
        }
        let key = self.key();
        g.spec_choices.iter().find(|c| c.card == me && c.key == key).copied()
    }
}

pub enum Flow {
    Next,
    /// Run nested list `sel` of the op (`ops::child`).
    Enter(u8),
    /// A prompt is open; `resume` continues.
    Suspend,
}

/// The spec being run for `me`: its own, or, while a copied attack runs the
/// source card's text as the copycat's (`copy_attack.rs` delegation), the
/// source's.
fn spec_of(g: &Game, me: CardId) -> &'static CardSpec {
    let card = match g.deleg {
        Some(d) if d.copycat == me => d.source,
        _ => me,
    };
    crate::cards::spec_for(g.st.cards[card as usize].def).expect("spec card without a spec")
}

fn program(spec: &'static CardSpec, prog: Prog) -> &'static [Step] {
    match prog {
        Prog::Attack(i) => spec.attacks[i as usize].steps,
        Prog::Play => spec.play.as_ref().map(|p| p.steps).unwrap_or(&[]),
        Prog::Power(i) => spec.powers[i as usize].steps,
        Prog::Trigger(i) => spec.triggers[i as usize].steps,
        Prog::UseStadium => spec.use_stadium.as_ref().map(|p| p.steps).unwrap_or(&[]),
    }
}

/// The list the frame's current level walks.
fn list_at(spec: &'static CardSpec, f: &Frame) -> &'static [Step] {
    let mut list = program(spec, f.prog);
    for level in 1..=f.depth as usize {
        let parent = &list[(f.path[level - 1] & INDEX_MASK) as usize];
        list = ops::child(&parent.op, f.path[level] >> SEL_SHIFT);
    }
    list
}

// ---------------------------------------------------------------------------
// Entry points

/// `CardImpl::reduce` of every spec card.
pub fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let spec = spec_of(g, me);
    for ps in spec.passives {
        passive::apply(g, me, e, ps)?;
    }
    for (i, a) in spec.attacks.iter().enumerate() {
        if was_attack_used(g, e, a.index, me) {
            if let Some((p, ..)) = attack_data(g, e) {
                g.spec_choices.retain(|c| c.card != me);
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
            let mut f = Frame::new(Prog::Play, Phase::Use, e, p);
            f.via_attack = trainer_via_attack(g, e);
            if !usable(g, me, &f, play.needs, play.steps)? {
                crate::bail!("CANNOT_PLAY_THIS_CARD");
            }
            run(g, me, f)?;
        }
    }
    for (i, pw) in spec.powers.iter().enumerate() {
        if let Once::PerTurn(name) | Once::PerTurnShared(name) = pw.once {
            remove_marker_at_end_of_turn(g, e, crate::markers::intern(name), me);
            if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
                if card == me {
                    g.st.players[p as usize].marker.remove_from(crate::markers::intern(name), me);
                }
            }
        }
        if was_power_used(g, e, pw.index, me) {
            let p = match *g.e(e) {
                Effect::Power { p, .. } => p as usize,
                _ => continue,
            };
            let f = Frame::new(Prog::Power(i as u8), Phase::Use, e, p);
            match pw.once {
                Once::PerTurn(name) => {
                    if g.st.players[p].marker.has_from(crate::markers::intern(name), me) {
                        crate::bail!("POWER_ALREADY_USED");
                    }
                }
                Once::PerTurnShared(name) => {
                    if g.st.players[p].marker.has(crate::markers::intern(name)) {
                        crate::bail!("POWER_ALREADY_USED");
                    }
                }
                Once::No => {}
            }
            if !usable(g, me, &f, pw.needs, pw.steps)? {
                crate::bail!("CANNOT_USE_POWER");
            }
            if let Once::PerTurn(name) | Once::PerTurnShared(name) = pw.once {
                use_ability_once_per_turn(g, p, crate::markers::intern(name), me)?;
                ability_used(g, p, me);
            }
            run(g, me, f)?;
        }
    }
    if let Some(us) = &spec.use_stadium {
        if let Effect::UseStadium { p, stadium } = *g.e(e) {
            if stadium == me {
                let f = Frame::new(Prog::UseStadium, Phase::Use, e, p as usize);
                if !usable(g, me, &f, us.needs, us.steps)? {
                    crate::bail!("CANNOT_USE_STADIUM");
                }
                run(g, me, f)?;
            }
        }
    }
    for (i, t) in spec.triggers.iter().enumerate() {
        // "When you play this Pokémon" runs once the card is on the board (`after_enter_play`).
        if trigger::runs_after_play(t) {
            continue;
        }
        if let Some((p, slot)) = trigger::fires(g, me, e, t) {
            let mut f = Frame::new(Prog::Trigger(i as u8), Phase::Use, e, p);
            f.slot = slot;
            if trigger::retains(t) {
                g.retain_fx(e);
            }
            run(g, me, f)?;
        }
    }
    Ok(())
}

/// The triggers of a Pokémon that was just played or evolved ("when you play this Pokémon from your
/// hand onto your Bench / to evolve"): the card is on the board now, so the ordinary in-play locks at its
/// slot decide whether its Ability triggers (RULES.md, On-play Abilities).
pub fn after_enter_play(g: &mut Game, e: EffId) -> R {
    let card = match *g.e(e) {
        Effect::PlayPokemon { card, .. } | Effect::Evolve { card, .. } => card,
        _ => return Ok(()),
    };
    let Some(spec) = crate::cards::spec_for(g.st.cards[card as usize].def) else { return Ok(()) };
    for (i, t) in spec.triggers.iter().enumerate() {
        if !trigger::runs_after_play(t) {
            continue;
        }
        if let Some((p, slot)) = trigger::fires(g, card, e, t) {
            let mut f = Frame::new(Prog::Trigger(i as u8), Phase::Use, e, p);
            f.slot = slot;
            run(g, card, f)?;
        }
    }
    Ok(())
}

/// A Trainer, Ability or Stadium can be used when its declared needs (read
/// as the game checks them: types, provided Energy) and the preconditions its
/// top-level ops imply hold.
fn usable(g: &mut Game, me: CardId, f: &Frame, needs: &[Cond], steps: &[Step]) -> R<bool> {
    for c in needs {
        if !cond_m(g, me, f, c)? {
            return Ok(false);
        }
    }
    Ok(steps.iter().all(|s| ops::implied_ok(g, me, f, &s.op)))
}

/// `CardImpl::resume` of every spec card.
pub fn resume(g: &mut Game, me: CardId, cf: CardFrame, results: &[Res]) -> R {
    if cf.stage == passive::HEAVY_BATON_STAGE {
        return passive::heavy_baton_resume(g, cf, results);
    }
    let Some(mut f) = Frame::decode(&cf) else { return Ok(()) };
    let spec = spec_of(g, me);
    let op = &list_at(spec, &f)[f.index()].op;
    let flow = if f.sub == COIN_SEQUENCE {
        // A finished coin sequence: the core passes the results (bit i = flip i was heads) and
        // the flip count; the frame is as the op left it.
        ops::resume_coin(g, me, &mut f, op, results)?
    } else if f.phase == Phase::Choices {
        ops::resume_choice(g, me, &mut f, op, results)?
    } else {
        ops::resume(g, me, &mut f, op, results)?
    };
    proceed(g, me, f, flow)
}

/// `CardImpl::coin` of every spec card: the result of a single flip.
pub fn coin(g: &mut Game, me: CardId, cf: CardFrame, heads: bool) -> R {
    let Some(mut f) = Frame::decode(&cf) else { return Ok(()) };
    let spec = spec_of(g, me);
    let op = &list_at(spec, &f)[f.index()].op;
    let flow = ops::coin_result(g, me, &mut f, op, heads)?;
    proceed(g, me, f, flow)
}

fn proceed(g: &mut Game, me: CardId, mut f: Frame, flow: Flow) -> R {
    match flow {
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
            if f.depth > 0 {
                // A nested list ended: a loop may run it again.
                let d = f.depth as usize;
                let mut parent_frame = f;
                parent_frame.depth -= 1;
                let parent = &list_at(spec, &parent_frame)[parent_frame.index()].op;
                f.iter[d - 1] += 1;
                if ops::again(g, me, &mut f, parent) {
                    f.path[d] &= !INDEX_MASK;
                    f.sub = 0;
                    continue;
                }
                f.iter[d - 1] = 0;
                f.depth -= 1;
                f.advance();
                continue;
            }
            match f.phase {
                // Step D follows the attack's before-damage text.
                Phase::BeforeDamage => {
                    f.phase = Phase::Choices;
                    f.path = [0; MAX_DEPTH];
                    f.sub = 0;
                    continue;
                }
                Phase::AfterDamage => g.spec_choices.retain(|c| c.card != me),
                _ => {}
            }
            if let Prog::Trigger(i) = f.prog {
                if trigger::retains(&spec.triggers[i as usize]) {
                    g.release_fx(f.eff);
                }
            }
            return Ok(());
        }
        let step = &list[i];
        if f.depth == 0 && !runs_in(step.at, f.phase) {
            f.advance();
            continue;
        }
        let flow = if f.phase == Phase::Choices { ops::choice(g, me, &mut f, &step.op)? } else { ops::exec(g, me, &mut f, &step.op)? };
        match flow {
            Flow::Next => f.advance(),
            Flow::Enter(sel) => f.enter(sel),
            Flow::Suspend => return Ok(()),
        }
    }
}

fn runs_in(at: RuleStep, phase: Phase) -> bool {
    matches!(
        (at, phase),
        (RuleStep::BeforeDamage, Phase::BeforeDamage)
            | (RuleStep::AfterDamage, Phase::AfterDamage)
            | (RuleStep::AfterDamage, Phase::Choices)
            | (RuleStep::Use, Phase::Use)
    )
}
