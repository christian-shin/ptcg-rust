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
use crate::cause::{Cause, CauseKind};
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
    /// Deck searches, and choices nested under an `If` the damage can change, a coin or a
    /// loop, are still made after the damage, when their options exist.
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
    /// "If you do": the last op that reports whether it did what it says did it (`Cond::Done`; events batch 5: the
    /// switch ops, true when the ChangeActive happened).
    pub(crate) done: bool,
    /// What the program's events are caused by (docs/design/events-design.md, section 3): set once when
    /// the frame is created (`frame_cause`), from what starts the program. Not encoded: a resumed frame
    /// computes it again from the same fixed fields.
    pub(crate) cause: Cause,
}

impl Frame {
    pub(crate) fn new(prog: Prog, phase: Phase, eff: EffId, p: usize, cause: Cause) -> Frame {
        Frame { prog, phase, path: [0; MAX_DEPTH], depth: 0, iter: [0; MAX_DEPTH], sub: 0, eff, p: p as u8, cards: [NONE; 2], heads: 0, slot: NONE, prize: NONE, attached_to: NONE, last: 0, via_attack: false, done: false, cause }
    }

    /// The frame a passive of card `me` (owned by `owner`) evaluates its conditions and numbers in. It runs no
    /// ops, so no event is made with its cause: the passive's origin (`Cause::of_origin`; a Pokémon's
    /// `CardRule` passive, the Tera rule, is not an Ability).
    pub(crate) fn passive(_g: &Game, me: CardId, owner: usize, origin: passive::RuleSource) -> Frame {
        Frame::new(Prog::Play, Phase::Use, 0, owner, Cause::of_origin(origin, me, owner as u8))
    }

    /// The frame of card `me`'s program `prog`, with its cause (`frame_cause`).
    pub(crate) fn start(g: &Game, me: CardId, prog: Prog, phase: Phase, eff: EffId, p: usize, via_attack: bool) -> Frame {
        let mut f = Frame::new(prog, phase, eff, p, frame_cause(g, me, prog, p as u8, eff, via_attack));
        f.via_attack = via_attack;
        f
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
            | (self.done as i32) << 6
            | (self.attached_to as i32) << 8
            | ((self.last & 0xFFFF) << 16);
        f.a[3] = i32::from_le_bytes(self.iter);
        f.e[0] = self.eff;
        f.e[1] = self.slot;
        f.l = self.cards;
        f
    }

    fn decode(g: &Game, me: CardId, f: &CardFrame) -> Option<Frame> {
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
            done: (f.a[2] >> 6) & 1 != 0,
            attached_to: ((f.a[2] >> 8) & 0xFF) as u8,
            last: (f.a[2] >> 16) as i16 as i32,
            cards: f.l,
            cause: Cause::rule(crate::cause::RuleWhich::Setup, 0),
        })
        .map(|mut fr| {
            fr.cause = frame_cause(g, me, fr.prog, fr.p, fr.eff, fr.via_attack);
            fr
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

/// The cause of card `me`'s program `prog` run for player `p` (events design, section 3), from what
/// starts it:
/// - an attack: `Attack`, the attacking Pokémon (`me`, also the copycat of a copied attack), the attack of
///   the frame's `AttackEffect`;
/// - a Trainer: its own kind (Item / Supporter: `Trainer`, Tool, Stadium); used as the effect of an attack
///   (Look-Alike Show): that attack, the Pokémon that used it;
/// - an Ability: `Ability`; the Stadium's use: `Stadium`;
/// - a trigger: the trigger's origin (`Cause::of_origin`), caused by the card's owner.
pub(crate) fn frame_cause(g: &Game, me: CardId, prog: Prog, p: u8, eff: EffId, via_attack: bool) -> Cause {
    match prog {
        Prog::Attack(_) => {
            let attack = if (eff as usize) < g.fx.len() { attack_data(g, eff).map(|d| d.2) } else { None };
            Cause { kind: CauseKind::Attack, card: Some(me), player: p, attack }
        }
        // The attack that uses the Trainer: the one of the AttackEffect being reduced (`Game::last_attack`,
        // set when the AttackEffect is dispatched; id2225, id2226, id2376).
        Prog::Play if via_attack => match g.last_attack.as_ref() {
            Some(l) => Cause { kind: CauseKind::Attack, card: l.pokemon, player: p, attack: Some(l.attack) },
            None => Cause { kind: CauseKind::Attack, card: None, player: p, attack: g.st.last_attack },
        },
        Prog::Play => Cause::of_trainer(g, me, p),
        Prog::Power(_) => Cause::new(CauseKind::Ability, Some(me), p),
        Prog::UseStadium => Cause::new(CauseKind::Stadium, Some(me), p),
        Prog::Trigger(i) => trigger_cause(g, me, &spec_of(g, me).triggers[i as usize], eff),
    }
}

/// A trigger's cause: its origin, caused by the card's owner; a `CardRule` trigger by its event (the end of
/// the turn's bookkeeping is `Rule { EndTurn }`; a trigger that continues an attack is that attack).
fn trigger_cause(g: &Game, me: CardId, t: &trigger::Trigger, eff: crate::effects::EffId) -> Cause {
    let owner = g.st.owner(me) as u8;
    if t.origin != passive::RuleSource::CardRule {
        return Cause::of_origin(t.origin, me, owner);
    }
    match &t.event {
        trigger::Event::OnEndTurn(_) => Cause::new(CauseKind::Rule { which: crate::cause::RuleWhich::EndTurn }, Some(me), owner),
        trigger::Event::OnAfterAttackTriggers(_) | trigger::Event::OnDamagedByAttack(_) => {
            let attack = if (eff as usize) < g.fx.len() {
                match *g.e(eff) {
                    Effect::AfterAttackTriggers { p, attack, .. } => Some(Cause { kind: CauseKind::Attack, card: g.last_attack.as_ref().and_then(|l| l.pokemon), player: p, attack: Some(attack) }),
                    Effect::AttackTrigger { p, attack, source, .. } => Some(Cause::of_attack_at(g, p, attack, source)),
                    _ => None,
                }
            } else {
                None
            };
            attack.unwrap_or_else(|| Cause::of_origin(t.origin, me, owner))
        }
        _ => Cause::of_origin(t.origin, me, owner),
    }
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
                run(g, me, Frame::start(g, me, Prog::Attack(i as u8), Phase::BeforeDamage, e, p as usize, false))?;
            }
        } else if after_attack_used(g, e, a.index, me) {
            let atk = real_attack(g, e);
            if let Some((p, ..)) = attack_data(g, atk) {
                run(g, me, Frame::start(g, me, Prog::Attack(i as u8), Phase::AfterDamage, atk, p as usize, false))?;
            }
        }
    }
    if let Some(play) = &spec.play {
        if let Some(p) = trainer_played(g, e, me) {
            let f = Frame::start(g, me, Prog::Play, Phase::Use, e, p, trainer_via_attack(g, e));
            check_play(g, me, play, &f)?;
            run(g, me, f)?;
        }
    }
    for (i, pw) in spec.powers.iter().enumerate() {
        if let Once::PerTurn(name) | Once::PerTurnShared(name) = pw.once {
            remove_marker_at_end_of_turn(g, e, crate::markers::intern(name), me);
            // A card that enters play or evolves is a new Pokémon (id317).
            if let Effect::EnterPlay { p, card, .. } | Effect::Evolve { p, card, .. } = *g.e(e) {
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
            let f = Frame::start(g, me, Prog::Power(i as u8), Phase::Use, e, p, false);
            check_power(g, me, pw, &f)?;
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
                let f = Frame::start(g, me, Prog::UseStadium, Phase::Use, e, p as usize, false);
                check_use_stadium(g, me, us, &f)?;
                run(g, me, f)?;
            }
        }
    }
    for (i, t) in spec.triggers.iter().enumerate() {
        if let Some((p, slot)) = trigger::fires(g, me, e, t) {
            let mut f = Frame::start(g, me, Prog::Trigger(i as u8), Phase::Use, e, p, false);
            f.slot = slot;
            run(g, me, f)?;
        }
    }
    Ok(())
}

/// The triggers over an event that is done (events batch 2: EnterPlay, Evolve, Devolve, Swap; batch 3: Attach,
/// MoveEnergy, MoveTool): the `Event::On` triggers of every card that declares one for the event's kind, in
/// propagation order. The event's
/// consequences are applied by now (a Pokémon is on the board, so the ordinary in-play locks at its slot decide
/// whether its Ability triggers; docs/rulings/RULES.md, On-play Abilities, and "Putting onto the Bench": Risky
/// Ruins after the Pokémon is on the Bench).
pub fn after_event(g: &mut Game, e: EffId) -> R {
    let kind = g.e(e).kind();
    if !crate::spec::event::EVENT_KINDS.has(kind) || g.prevented(e) {
        return Ok(());
    }
    if g.kinds_present.has(kind) {
        let order = g.listeners(crate::game::prop_class(g.e(e)), kind);
        for &c in order.iter() {
            let Some(spec) = crate::cards::spec_for(g.st.cards[c as usize].def) else { continue };
            for (i, t) in spec.triggers.iter().enumerate() {
                if !matches!(t.event, trigger::Event::On(_)) {
                    continue;
                }
                if let Some((p, slot)) = trigger::fires_on(g, c, e, t)? {
                    let mut f = Frame::start(g, c, Prog::Trigger(i as u8), Phase::Use, e, p, false);
                    f.slot = slot;
                    run(g, c, f)?;
                }
            }
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

/// The declared checks of playing a Trainer (`reduce` and legality both make them): the one-Supporter rule
/// and the card's `needs` and implied preconditions.
pub(crate) fn check_play(g: &mut Game, me: CardId, play: &PlaySpec, f: &Frame) -> R {
    if play.kind == PlayKind::Supporter && g.st.players[f.p as usize].supporter_turn > 0 {
        crate::bail!("SUPPORTER_ALREADY_PLAYED");
    }
    if !usable(g, me, f, play.needs, play.steps)? {
        crate::bail!("CANNOT_PLAY_THIS_CARD");
    }
    Ok(())
}

/// The declared checks of using an Ability: once per turn, then its `needs` and implied preconditions.
pub(crate) fn check_power(g: &mut Game, me: CardId, pw: &PowerSpec, f: &Frame) -> R {
    check_once(g, me, pw, f.p as usize)?;
    if !usable(g, me, f, pw.needs, pw.steps)? {
        crate::bail!("CANNOT_USE_POWER");
    }
    Ok(())
}

/// Once per turn: refused while the marker is set.
fn check_once(g: &Game, me: CardId, pw: &PowerSpec, p: usize) -> R {
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
    Ok(())
}

/// The declared checks of using the Stadium in play.
pub(crate) fn check_use_stadium(g: &mut Game, me: CardId, us: &PlaySpec, f: &Frame) -> R {
    if !usable(g, me, f, us.needs, us.steps)? {
        crate::bail!("CANNOT_USE_STADIUM");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Legality: the declared checks of a card, evaluated without running its program. `e` is the effect the
// use would carry (made by the caller on its scratch game); each returns `Ok` for a card that declares none.

/// A Trainer's play checks (`trainer_played`'s card, player `p`).
pub fn trainer_play_check(g: &mut Game, me: CardId, p: usize, e: EffId) -> R {
    let Some(spec) = crate::cards::spec_for(g.st.cards[me as usize].def) else { return Ok(()) };
    let Some(play) = &spec.play else { return Ok(()) };
    let f = Frame::start(g, me, Prog::Play, Phase::Use, e, p, trainer_via_attack(g, e));
    check_play(g, me, play, &f)
}

/// Legality, the cheap half of the power checks (read from the game as it is): is the once-per-turn marker free?
pub fn power_once_check(g: &Game, me: CardId, index: u8, p: usize) -> R {
    let Some(spec) = crate::cards::spec_for(g.st.cards[me as usize].def) else { return Ok(()) };
    match spec.powers.iter().find(|pw| pw.index == index) {
        Some(pw) => check_once(g, me, pw, p),
        None => Ok(()),
    }
}

/// The checks of using the power with printed index `index` of `me`.
pub fn power_check(g: &mut Game, me: CardId, index: u8, p: usize, e: EffId) -> R {
    let Some(spec) = crate::cards::spec_for(g.st.cards[me as usize].def) else { return Ok(()) };
    for (i, pw) in spec.powers.iter().enumerate() {
        if pw.index == index {
            let f = Frame::start(g, me, Prog::Power(i as u8), Phase::Use, e, p, false);
            return check_power(g, me, pw, &f);
        }
    }
    Ok(())
}

/// The checks of using the Stadium `me` in play.
pub fn use_stadium_check(g: &mut Game, me: CardId, p: usize, e: EffId) -> R {
    let Some(spec) = crate::cards::spec_for(g.st.cards[me as usize].def) else { return Ok(()) };
    let Some(us) = &spec.use_stadium else { return Ok(()) };
    let f = Frame::start(g, me, Prog::UseStadium, Phase::Use, e, p, false);
    check_use_stadium(g, me, us, &f)
}

/// What an attack's declared preconditions (its leading `Op::Fail` steps) say.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gate {
    Open,
    Closed,
    /// A `Fail` the leading steps don't cover (after other steps, or nested): not declared as a precondition.
    Undeclared,
}

fn has_fail(steps: &[Step]) -> bool {
    steps.iter().any(|s| matches!(s.op, Op::Fail(_)) || (0..=8).any(|sel| has_fail(ops::child(&s.op, sel))))
}

/// Does any attack of the card have a `Fail` op (legality reads its leading ones as preconditions)?
pub fn spec_has_fail(spec: &CardSpec) -> bool {
    spec.attacks.iter().any(|a| has_fail(a.steps))
}

/// Does the Trainer's play put the card onto the Bench as a Pokémon (`PlayAsPokemon`, the Antique Fossils)?
/// Legality then also asks the checks of playing a Pokémon from the hand.
pub fn plays_as_pokemon(spec: &CardSpec) -> bool {
    spec.play.as_ref().map_or(false, |p| p.steps.iter().any(|s| matches!(s.op, Op::PlayAsPokemon(_))))
}

/// The leading `Fail` steps of attack `attack`'s text before the damage (`e`: the `Attack` effect the use
/// would carry; `p` the attacking player; `me` the card whose program runs: the attack's card, or the
/// copycat when the attack is copied from a Benched Pokémon), evaluated as `run` does.
pub fn attack_gate(g: &mut Game, me: CardId, attack: crate::state::AttackRef, p: usize, e: EffId) -> R<Gate> {
    let Some(spec) = crate::cards::spec_for(g.st.cards[attack.card as usize].def) else { return Ok(Gate::Open) };
    let Some((i, a)) = spec.attacks.iter().enumerate().find(|(_, a)| a.index == attack.idx() as u8) else { return Ok(Gate::Open) };
    let f = Frame::start(g, me, Prog::Attack(i as u8), Phase::BeforeDamage, e, p, false);
    let mut leading = true;
    for s in a.steps {
        match &s.op {
            Op::Fail(x) if leading && s.at == RuleStep::BeforeDamage => match ops::fail_holds(g, me, &f, x) {
                Ok(true) => {}
                _ => return Ok(Gate::Closed),
            },
            op => {
                if s.at == RuleStep::BeforeDamage {
                    leading = false;
                }
                if matches!(op, Op::Fail(_)) || (0..=8).any(|sel| has_fail(ops::child(op, sel))) {
                    return Ok(Gate::Undeclared);
                }
            }
        }
    }
    Ok(Gate::Open)
}

/// `CardImpl::resume` of every spec card.
pub fn resume(g: &mut Game, me: CardId, cf: CardFrame, results: &[Res]) -> R {
    let Some(mut f) = Frame::decode(g, me, &cf) else { return Ok(()) };
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
    let Some(mut f) = Frame::decode(g, me, &cf) else { return Ok(()) };
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
