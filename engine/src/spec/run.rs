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
use crate::engine::order;
use crate::effects::{AtkBase, EffId, Effect};
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
    /// loop, are still made after the damage, when their options exist. At a table with
    /// `TableRules::attack_choices_asked_when_their_effect_runs` only the choices the damage reads (`damage_reads`).
    Choices = 3,
    /// The consequences of using an Ability (`Step::consequence`: "If you use this Ability, this Pokémon is Knocked
    /// Out"), done when it is used, before the rest of its text; the program goes on with `AfterConsequences`.
    Consequences = 4,
    /// The rest of such an Ability (its costs and `Step::new` steps, in printed order), after its consequences (the
    /// Pokémon's Knock Out pending; the state check after the effect handles it).
    AfterConsequences = 5,
}

/// What the last op that makes events did (events design section 7; APR II.A "do as much as you can", E-20): `Done` it
/// did all it says, `Partial` some of it (fewer cards than asked, a card refused), `Prevented` nothing because every
/// event was refused (a lock, a prevention), `Impossible` nothing because there was nothing to do it to. "If you do"
/// (`Cond::Done`) is `Done | Partial` (E-20: having done part of the first half is enough); "if you can't ... (in full)"
/// (`Cond::DidAll`) reads `Done`. Ops that make no event leave it as it was.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Done = 0,
    Partial = 1,
    Prevented = 2,
    Impossible = 3,
}

impl Outcome {
    /// "If you do": the op did at least part of what it says.
    #[inline]
    pub const fn did(self) -> bool {
        matches!(self, Outcome::Done | Outcome::Partial)
    }

    /// One event that happened or was refused.
    #[inline]
    pub const fn of(happened: bool) -> Outcome {
        if happened {
            Outcome::Done
        } else {
            Outcome::Prevented
        }
    }

    /// `done` of `asked` cards moved (a refused one stays where it was): Done, Partial, Prevented (some asked, none moved
    /// because each was refused) or Impossible (nothing asked).
    pub const fn count(asked: usize, done: usize, refused: usize) -> Outcome {
        if asked == 0 {
            Outcome::Impossible
        } else if done >= asked {
            Outcome::Done
        } else if done > 0 {
            Outcome::Partial
        } else if refused > 0 {
            Outcome::Prevented
        } else {
            Outcome::Impossible
        }
    }

    const fn from_bits(b: i32) -> Outcome {
        match b & 3 {
            0 => Outcome::Done,
            1 => Outcome::Partial,
            2 => Outcome::Prevented,
            _ => Outcome::Impossible,
        }
    }
}

/// A program's position and registers, round-tripped through `CardFrame`.
///
/// Layout (`CardFrame { a: [i32; 4], e: [u8; 2], l: [u8; 2] }`):
/// - `a[0]`: bits 0-15 program code, 16-19 depth, 20-23 heads, 24-31 resume point (`sub`);
/// - `a[1]`: the path, one byte per level;
/// - `a[2]`: bit 0 player `p`, 1-4 prize pile (15: none), 5 `via_attack`, 6-7 `outcome`,
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
    /// The Pokémon the last `Attach` attached cards to (`p << 4 | slot`; the Active Pokémon when one of the cards went
    /// there), or NONE.
    pub(crate) attached_to: u8,
    /// Cards the last discard of the program moved (`Num::Last`, 16 bits).
    pub(crate) last: i32,
    /// The Trainer is used as the effect of an attack (Look-Alike Show); fixed when it starts.
    pub(crate) via_attack: bool,
    /// What the last op that makes events did (`Cond::Done`, `Cond::DidAll`; events batch 7). `Impossible` until one
    /// reports.
    pub(crate) outcome: Outcome,
    /// What the program's events are caused by (docs/design/events-design.md, section 3): set once when
    /// the frame is created (`frame_cause`), from what starts the program. Not encoded: a resumed frame
    /// computes it again from the same fixed fields.
    pub(crate) cause: Cause,
    /// The cards out of the hand while the program resolves, for a legality check made before the play (`for_check`: a
    /// played Item and its companion are in the play area); `NO_CARD` otherwise. Not encoded (a check suspends nothing).
    pub(crate) excl: [CardId; 2],
}

/// The effect of a frame that has none in the arena (a legality check's, `Frame::for_check`).
pub(crate) const NO_EFF: EffId = EffId::MAX;

impl Frame {
    pub(crate) fn new(prog: Prog, phase: Phase, eff: EffId, p: usize, cause: Cause) -> Frame {
        Frame { prog, phase, path: [0; MAX_DEPTH], depth: 0, iter: [0; MAX_DEPTH], sub: 0, eff, p: p as u8, cards: [NONE; 2], heads: 0, slot: NONE, prize: NONE, attached_to: NONE, last: 0, via_attack: false, outcome: Outcome::Impossible, cause, excl: [NO_CARD; 2] }
    }

    /// The frame legality reads a card's checks in, before the card is used (batch 8 C3d: no scratch game): no effect in
    /// the arena (`NO_EFF`), the cause the program would have, and the cards out of the hand while it resolves (`excl`).
    pub(crate) fn for_check(prog: Prog, phase: Phase, p: usize, cause: Cause, excl: [CardId; 2]) -> Frame {
        let mut f = Frame::new(prog, phase, NO_EFF, p, cause);
        f.excl = excl;
        f
    }

    /// Is card `c` out of the hand for this program (a legality check's played Item or its companion)?
    #[inline]
    pub(crate) fn out_of_hand(&self, c: CardId) -> bool {
        c != NO_CARD && (self.excl[0] == c || self.excl[1] == c)
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
            | (self.outcome as i32) << 6
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
            4 => Phase::Consequences,
            5 => Phase::AfterConsequences,
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
            outcome: Outcome::from_bits(f.a[2] >> 6),
            attached_to: ((f.a[2] >> 8) & 0xFF) as u8,
            last: (f.a[2] >> 16) as i16 as i32,
            cards: f.l,
            cause: Cause::rule(crate::cause::RuleWhich::Setup, 0),
            excl: [NO_CARD; 2],
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
/// Does the power's program have a consequence Knocking this Pokémon Out ("If you use this Ability, this Pokémon is
/// Knocked Out": Cursed Blast, Overvolt Discharge)? Psyduck's Damp removes the Abilities that require it.
pub fn power_knocks_out_self(g: &Game, power: crate::effects::PowerRef) -> bool {
    if power.card as usize >= crate::state::MAX_CARDS {
        return false;
    }
    let Some(spec) = crate::cards::spec_for(g.st.cards[power.card as usize].def) else { return false };
    spec.powers.iter().filter(|pw| pw.index == power.index).any(program_knocks_out_self)
}

/// Does the program have a consequence Knock Out of this Pokémon?
pub fn program_knocks_out_self(pw: &PowerSpec) -> bool {
    pw.steps.iter().any(|s| s.at == RuleStep::Consequence && matches!(&s.op, Op::KnockOut(k) if k.target == SlotExpr::This))
}

/// Does the program have a consequence of using it (`Step::consequence`), done when it is used?
fn program_has_consequence(pw: &PowerSpec) -> bool {
    pw.steps.iter().any(|s| s.at == RuleStep::Consequence)
}

/// Does the power have a program (a `PowerSpec`): would a `Power` effect for it run anything?
pub fn power_has_program(g: &Game, power: crate::effects::PowerRef) -> bool {
    if power.card as usize >= crate::state::MAX_CARDS {
        return false;
    }
    crate::cards::spec_for(g.st.cards[power.card as usize].def).map_or(false, |s| s.powers.iter().any(|pw| pw.index == power.index))
}

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

/// The steps of the program the frame runs (of the card `me`, or of the card whose attack `me` copies).
pub(crate) fn program_steps(g: &Game, me: CardId, f: &Frame) -> &'static [Step] {
    program(spec_of(g, me), f.prog)
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
///   the frame's attack effect;
/// - a Trainer: its own kind (Item / Supporter: `Trainer`, Tool, Stadium); used as the effect of an attack
///   (Look-Alike Show): that attack, the Pokémon that used it;
/// - an Ability: `Ability` (a power that is a rule printed on the card, a Fossil's discard: `Rule { CardRule }`); the Stadium's use: `Stadium`;
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
        Prog::Power(i) => power_cause(g, me, i, p),
        Prog::UseStadium => Cause::new(CauseKind::Stadium, Some(me), p),
        Prog::Trigger(i) => trigger_cause(g, me, &spec_of(g, me).triggers[i as usize], eff),
    }
}

/// The cause of using the declared power `i` of `me` (player `p`): an Ability's is `Ability`; a power that is a rule printed
/// on the card (a Fossil's "you may discard this card from play") is `Rule { CardRule }` (`PowerDef::source`), so what
/// locks or prevents Abilities doesn't reach it.
fn power_cause(g: &Game, me: CardId, i: u8, p: u8) -> Cause {
    let printed = spec_of(g, me).powers[i as usize].index as usize;
    Cause::of_origin(g.st.cdef(me).powers[printed].source(), me, p)
}

/// A trigger's cause: its origin, caused by the card's owner; a `CardRule` trigger by its event (the end of
/// the turn's bookkeeping is `Rule { EndTurn }`).
fn trigger_cause(g: &Game, me: CardId, t: &trigger::Trigger, eff: crate::effects::EffId) -> Cause {
    let owner = g.st.owner(me) as u8;
    if t.origin != passive::RuleSource::CardRule {
        return Cause::of_origin(t.origin, me, owner);
    }
    match &t.event {
        // The end of the turn's bookkeeping of a card's own text (`Rule { EndTurn }`).
        trigger::Event::On(_) if (eff as usize) < g.fx.len() && matches!(*g.e(eff), Effect::EndTurn { .. }) => Cause::new(CauseKind::Rule { which: crate::cause::RuleWhich::EndTurn }, Some(me), owner),
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
        if was_power_used(g, e, pw.index, me) {
            let p = match *g.e(e) {
                Effect::Power { p, .. } => p as usize,
                _ => continue,
            };
            // The consequences of using it come first (`Phase::Consequences`), then the rest of the Ability.
            let phase = if program_has_consequence(pw) { Phase::Consequences } else { Phase::Use };
            let f = Frame::start(g, me, Prog::Power(i as u8), phase, e, p, false);
            check_power(g, me, pw, &f)?;
            if let Some((key, by, until)) = power_once(g, me, pw, p) {
                if !crate::engine::usage::use_once(&mut g.st, key, by, until) {
                    crate::bail!("POWER_ALREADY_USED");
                }
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
    Ok(())
}

/// The triggers over an event that is done (events batch 2: EnterPlay, Evolve, Devolve, Swap; batch 3: Attach,
/// MoveEnergy, MoveTool): the `Event::On` triggers of every card that declares one for the event's kind, in
/// propagation order. The event's
/// consequences are applied by now (a Pokémon is on the board, so the ordinary in-play locks at its slot decide
/// whether its Ability triggers; docs/rulings/RULES.md, On-play Abilities, and "Putting onto the Bench": Risky
/// Ruins after the Pokémon is on the Bench). The triggers over the Pokémon an effect puts into play wait until that effect
/// is done (`put_window`).
pub fn after_event(g: &mut Game, e: EffId) -> R {
    let kind = g.e(e).kind();
    if !crate::spec::event::TRIGGER_EVENT_KINDS.has(kind) || g.prevented(e) {
        return Ok(());
    }
    // The triggers of the Knock Outs of one check are applied together in step 2 of the Knock Out process, in the order
    // the player whose turn it is picks (APR E-04): the state check's window collects them (`knockout::ko_effects`).
    if kind == crate::effects::k::KNOCK_OUT {
        return Ok(());
    }
    if !g.kinds_present.has(kind) {
        return Ok(());
    }
    // An event carrying several cards (events batch 7, user decision D3) is seen per card: "when this card is discarded"
    // fires once for that card.
    let cards: Option<crate::list::SVec<CardId, 64>> = crate::engine::cards_zone::event_cards(g.e(e)).copied();
    match cards {
        Some(cs) => {
            for &card in cs.iter() {
                fire_on(g, e, kind, Some(card))?;
            }
        }
        None => fire_on(g, e, kind, None)?,
    }
    Ok(())
}

/// A trigger about to fire at the end of a turn or at Pokémon Checkup: its card, its index, its player and spot, and the
/// event it fires on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timed {
    pub c: CardId,
    pub i: u8,
    pub p: u8,
    pub slot: u8,
    pub e: EffId,
}

/// The triggers of the event `e` (a turn-structure event, a Knock Out) that fire now, in the propagation order, and whether
/// each is an effect that applies (APR E-04, E-21, F step 2: "several such effects"; `applies`).
pub(crate) fn timed_triggers(g: &mut Game, e: EffId, kind: u32) -> R<SVec<(Timed, bool), 32>> {
    let class = crate::game::prop_class(g.e(e));
    let order = g.decl_ordered(g.decls.kind(kind), class);
    let mut out: SVec<(Timed, bool), 32> = SVec::new();
    let Some(v) = trigger::event_view(g, e) else { return Ok(out) };
    for &c in order.iter() {
        let Some(spec) = crate::cards::spec_for(g.st.cards[c as usize].def) else { continue };
        for (i, t) in spec.triggers.iter().enumerate() {
            if !matches!(t.event, trigger::Event::On(_)) {
                continue;
            }
            if let Some((p, slot)) = trigger::fires_on(g, c, &v, t)? {
                let mut f = Frame::start(g, c, Prog::Trigger(i as u8), Phase::Use, e, p, false);
                f.slot = slot;
                let effect = applies(g, c, &f, t.steps);
                assert!(out.len() < out.capacity(), "more than {} triggers of one turn-structure event", out.capacity());
                out.push((Timed { c, i: i as u8, p: p as u8, slot, e }, effect));
            }
        }
    }
    Ok(out)
}

/// Fire one turn-structure trigger (the order prompt's choice), if it still fires.
pub(crate) fn fire_timed(g: &mut Game, t: Timed) -> R {
    fire_trigger(g, t.c, t.i, t.e)
}

/// Fire trigger `i` of card `c` over the event `e`, if it still fires (as the board is now).
fn fire_trigger(g: &mut Game, c: CardId, i: u8, e: EffId) -> R {
    let Some(spec) = crate::cards::spec_for(g.st.cards[c as usize].def) else { return Ok(()) };
    let Some(tr) = spec.triggers.get(i as usize) else { return Ok(()) };
    let Some(v) = trigger::event_view(g, e) else { return Ok(()) };
    if let Some((p, slot)) = trigger::fires_on(g, c, &v, tr)? {
        let mut f = Frame::start(g, c, Prog::Trigger(i), Phase::Use, e, p, false);
        f.slot = slot;
        run(g, c, f)?;
    }
    Ok(())
}

/// The attack's E-03 window while the Damage `e` is done in it: the effects that activate when a Pokémon is damaged by an
/// attack apply after the attack's damage and effects (APR E-03), whatever card declares them. Damage outside a window
/// (no step 7 open) is applied at once like any event.
fn damaged_window(g: &Game, e: EffId) -> Option<(EffId, AtkBase, i32)> {
    match *g.e(e) {
        Effect::Damage { b, amount, .. } if g.triggers_open(b.attack_effect) => Some((b.attack_effect, b, amount)),
        _ => None,
    }
}

/// The put-into-play window of the effect putting Pokémon into play while the EnterPlay `e` is done (`Game::put_by`): the
/// effects activated by Pokémon an effect puts into play apply once that effect is done, all of them on the Bench (JP FAQ
/// Risky Ruins x Call for Family x Solrock / Lunatone with New Moon; `order::after_effect`). The trigger `i` of card `c`
/// for player `p`, as recorded.
fn put_window(g: &Game, e: EffId, c: CardId, i: u8, p: u8) -> Option<(CardId, order::EnteredTrigger)> {
    let by = g.put_by?;
    match *g.e(e) {
        Effect::EnterPlay { card, target, from, source, cause, .. } => Some((by, order::EnteredTrigger { c, i, p, card, target, from, source, cause })),
        _ => None,
    }
}

/// The EnterPlay event a put-into-play window's trigger was recorded on, made again as it was.
fn entered_event(t: &order::EnteredTrigger) -> Effect {
    Effect::EnterPlay { p: t.target.p, card: t.card, target: t.target, from: t.from, source: t.source, mode: super::event::EnterMode::Effect, cause: t.cause }
}

/// Apply a trigger recorded in the put-into-play window: the EnterPlay event made again as it was, and the trigger re-checked
/// against the board now (`fires_on`).
pub(crate) fn fire_entered(g: &mut Game, t: &order::EnteredTrigger) -> R {
    fire_recorded(g, t.c, t.i, entered_event(t))
}

/// Fire trigger `i` of card `c` recorded on the event `ev`, made again as it was, if it still fires.
fn fire_recorded(g: &mut Game, c: CardId, i: u8, ev: Effect) -> R {
    let e = g.new_fx(ev);
    let r = fire_trigger(g, c, i, e);
    g.release_fx(e);
    r
}

/// Apply a trigger recorded in the E-03 window: the Damage event made again as it was (`DamagedTrigger`), and the trigger
/// re-checked against the board now (still attached, not blocked: `fires_on`).
pub(crate) fn fire_damaged(g: &mut Game, d: &order::DamagedTrigger) -> R {
    fire_recorded(g, d.c, d.i, Effect::Damage { b: d.b, amount: d.amount, survive: false })
}

/// The `Event::On` triggers over one view of the event `e` (the view of `card` of an event carrying cards), in the
/// propagation order of the declaring cards. The view is read for each trigger, as the board is when it is asked.
/// The end of a turn with two or more distinct effects that apply: the turn player orders them (APR E-21; batch 8 C11c, user
/// decision 7; `engine::order`). At Pokémon Checkup the routine orders them (`phase`); here only the triggers it doesn't
/// hold back fire.
fn fire_on(g: &mut Game, e: EffId, kind: u32, card: Option<CardId>) -> R {
    if card.is_none() && kind == crate::effects::k::END_TURN {
        let mut items: SVec<(order::Item, bool), 32> = SVec::new();
        for (t, x) in timed_triggers(g, e, kind)?.iter() {
            items.push((order::Item::Card(*t), *x));
        }
        // Asked only with two or more distinct effects (the order of identical ones can't change the game).
        if order::asks(g, items.as_slice()) {
            let Effect::EndTurn { p } = *g.e(e) else { return Ok(()) };
            g.retain_fx(e);
            order::open(g, order::Window::EndTurn { p, e }, items.as_slice())?;
            g.order_pending = true;
            return Ok(());
        }
    }
    if card.is_none() && kind == crate::effects::k::BETWEEN_TURNS && g.checkup_held {
        // The Pokémon Checkup routine orders the effects (two or more of them apply): only the others fire here.
        let timed = timed_triggers(g, e, kind)?;
        for (t, x) in timed.iter().copied() {
            if !x {
                fire_timed(g, t)?;
            }
        }
        return Ok(());
    }
    let class = crate::game::prop_class(g.e(e));
    let order = g.decl_ordered(g.decls.kind(kind), class);
    for &c in order.iter() {
        let Some(spec) = crate::cards::spec_for(g.st.cards[c as usize].def) else { continue };
        for (i, t) in spec.triggers.iter().enumerate() {
            if !matches!(t.event, trigger::Event::On(_)) {
                continue;
            }
            let v = match card {
                Some(x) => crate::engine::cards_zone::view_of(g, g.e(e), x),
                None => trigger::event_view(g, e),
            };
            let Some(v) = v else { continue };
            if let Some((p, slot)) = trigger::fires_on(g, c, &v, t)? {
                if let Some((atk, b, amount)) = damaged_window(g, e) {
                    order::record(g, order::Tag::Damaged(atk), order::Item::Damaged(order::DamagedTrigger { c, i: i as u8, b, amount }));
                    continue;
                }
                if held_until_trainer_done(g, e, kind) {
                    g.retain_fx(e);
                    order::record(g, order::Tag::TrainerDone, order::Item::Held(Timed { c, i: i as u8, p: p as u8, slot, e }));
                    continue;
                }
                if let Some((by, entered)) = put_window(g, e, c, i as u8, p as u8) {
                    order::record(g, order::Tag::PutIntoPlay(by), order::Item::Entered(entered));
                    continue;
                }
                let mut f = Frame::start(g, c, Prog::Trigger(i as u8), Phase::Use, e, p, false);
                f.slot = slot;
                run(g, c, f)?;
            }
        }
    }
    Ok(())
}

/// Under the table rule `TableRules::evolving_trainer_discarded_before_evolve_triggers`, is the Evolve `e` (of kind `kind`)
/// caused by an Item or Supporter card still being played (in its player's play area), so its triggers wait until the
/// card is done and discarded (`engine::play_trainer::trainer_cleanup`)?
fn held_until_trainer_done(g: &Game, e: EffId, kind: u32) -> bool {
    if !g.st.rules.evolving_trainer_discarded_before_evolve_triggers || kind != crate::effects::k::EVOLVE {
        return false;
    }
    match g.e(e).cause() {
        Some(Cause { kind: CauseKind::Trainer, card: Some(t), .. }) => g.st.players[g.st.owner(t)].supporter().contains(t),
        _ => false,
    }
}

/// A trigger held until a Trainer was done (`held_until_trainer_done`), fired now (if it still fires); the event it was
/// held with is released.
pub(crate) fn fire_held(g: &mut Game, t: Timed) -> R {
    let r = fire_timed(g, t);
    g.release_fx(t.e);
    r
}

/// A Trainer, Ability or Stadium can be used when its declared `needs` hold (its printed conditions, and what the rules'
/// walk doesn't derive yet: types, provided Energy, checked reads) and the rules' usability holds (`usable_by_rules`; user
/// decision D10).
fn usable<F: crate::derived::Facts>(g: &mut F, me: CardId, f: &Frame, needs: &[Cond], steps: &[Step]) -> R<bool> {
    for c in needs {
        if !cond_m(g, me, f, c)? {
            return Ok(false);
        }
    }
    Ok(usable_by_rules(g.g(), me, f, steps))
}

/// The rules' usability of a Trainer's or an Ability's program (events design section 7; APR A-02, B-01, D-02, E-20, E-30;
/// id925, id956: "you cannot play a Trainer card for no net effect"; user decision D10), the card's `needs` aside (checked
/// first by `usable`): every cost can be paid (`Step::cost`), every choice the program starts with has something to choose
/// (a Pick, a PickSlot: the program's gates), and at least one of its effects would change the game (an op under
/// `If(Done)` counts only through the op it depends on: E-20's "If you do"). A program with no effect op is usable.
pub(crate) fn usable_by_rules(g: &Game, me: CardId, f: &Frame, steps: &[Step]) -> bool {
    RulesWalk::new(false).usable(g, me, f, steps)
}

/// Could the "yes" of a "you may" change the game (APR B-01; id925, id956: the rules' walk of `usable_by_rules` over the
/// branch: an effect op that would do something, a coin flip through its results)? A branch with no effect op can (it is
/// asked). Events batch 8 C10 (user decision 2a): a "you may" whose yes can't change the game isn't asked.
pub(crate) fn branch_can_change(g: &Game, me: CardId, f: &Frame, steps: &[Step]) -> bool {
    let mut w = RulesWalk::new(false);
    w.walk(g, me, f, steps, false) != Some(false)
}

/// Does an effect's program apply now: could it change the game (APR E-03, E-04, E-21, F step 2: the order of simultaneous
/// effects is asked only among effects that do; RULES.md "The order of simultaneous effects")? The walk of `usable_by_rules`
/// and `branch_can_change` over the program as it would run now: a condition on the board decides its branch, and a program
/// with no effect op (record bookkeeping) or whose every effect op would do nothing doesn't apply.
pub(crate) fn applies(g: &Game, me: CardId, f: &Frame, steps: &[Step]) -> bool {
    RulesWalk::new(true).walk(g, me, f, steps, false) == Some(true)
}

/// Does a trigger recorded in the E-03 window apply now (APR E-03: "the effects activating when a Pokémon is damaged"): it
/// fires on the Damage as the board is now and its program can change the game (`applies`).
pub(crate) fn damaged_applies(g: &mut Game, d: &order::DamagedTrigger) -> bool {
    recorded_applies(g, d.c, d.i, Effect::Damage { b: d.b, amount: d.amount, survive: false })
}

/// Does a trigger recorded in the put-into-play window apply now: it fires on the EnterPlay as the board is now and its
/// program can change the game (`applies`).
pub(crate) fn entered_applies(g: &mut Game, t: &order::EnteredTrigger) -> bool {
    recorded_applies(g, t.c, t.i, entered_event(t))
}

/// Does trigger `i` of card `c`, recorded on the event `ev` (made again as it was), apply now?
fn recorded_applies(g: &mut Game, c: CardId, i: u8, ev: Effect) -> bool {
    let Some(spec) = crate::cards::spec_for(g.st.cards[c as usize].def) else { return false };
    let Some(tr) = spec.triggers.get(i as usize) else { return false };
    let e = g.new_fx(ev);
    let mut yes = false;
    if let Some(v) = trigger::event_view(g, e) {
        if let Ok(Some((p, slot))) = trigger::fires_on(g, c, &v, tr) {
            let mut f = Frame::start(g, c, Prog::Trigger(i), Phase::Use, e, p, false);
            f.slot = slot;
            yes = applies(g, c, &f, tr.steps);
        }
    }
    g.release_fx(e);
    yes
}

/// A condition read as the board is now, checked reads included (`None`: the read fails).
fn cond_now(g: &Game, me: CardId, f: &Frame, c: &Cond) -> Option<bool> {
    crate::spec::value::cond_m(&mut crate::legal::Ctx::new(g), me, f, c).ok()
}

/// The walk of `usable_by_rules`: the cards the program takes from its player's hand so far (they leave it: "draw until
/// you have 6 cards" counts after the discard), and whether every top-level cost and choice can be made.
struct RulesWalk {
    /// The program runs now (`applies`): a condition on the board decides its branch; otherwise (a usability walk, a
    /// "you may" asked before the program's earlier ops ran) only the conditions that stay the same through the program do.
    now: bool,
    /// The question of `TableRules::hide_attacks_that_can_do_nothing` (`attack_can_do_something`): has every effect of
    /// the text something to act on, rather than would one change the game (`ops::has_object`)? An effect on a Pokémon
    /// the text names acts on it while it is in play, whether or not it would change it ("Heal 10 damage from this
    /// Pokémon" at full HP); a deck is hidden, so a pick or search from it has something to act on while it holds a
    /// card, its cards' kinds and the copies in known areas unread. A condition the text reads before it does anything
    /// ("If you have 13 or more ...", "If your opponent's Active Pokémon has exactly 6 damage counters on it") is read
    /// as the board is now; the text can do something only when every effect it does has something to act on.
    objects: bool,
    /// The program did something before the step walked (flipped a coin, chose, moved a card: any op but an "If" and
    /// the attack's damage), so its conditions aren't read as the board is now (`objects`).
    acted: bool,
    left_hand: i32,
    gates_ok: bool,
    /// Per card register: whether the op that fills it would find cards (`None`: not filled by the program yet).
    regs: [Option<bool>; 4],
}

impl RulesWalk {
    fn new(now: bool) -> Self {
        RulesWalk { now, objects: false, acted: false, left_hand: 0, gates_ok: true, regs: [None; 4] }
    }
    /// The program can be used: every top-level cost and choice can be made and no effect is known to do nothing (under
    /// `objects`: and an effect has something to act on).
    fn usable(&mut self, g: &Game, me: CardId, f: &Frame, steps: &[Step]) -> bool {
        let fx = self.walk(g, me, f, steps, true);
        self.gates_ok && if self.objects { fx == Some(true) } else { fx != Some(false) }
    }
    /// Can the op's choice or precondition be made (`ops::implied_ok`; under `objects` a pick from a deck can while the
    /// deck holds a card: `ops::has_object`)?
    fn implied_ok(&self, g: &Game, me: CardId, f: &Frame, op: &Op) -> bool {
        match self.objects.then(|| ops::has_object(g, me, f, op, false)).flatten() {
            Some(x) => x,
            None => ops::implied_ok(g, me, f, op),
        }
    }
    fn reg(&self, r: u8) -> Option<bool> {
        self.regs.get(r as usize).copied().flatten()
    }
    fn set_reg(&mut self, r: u8, v: bool) {
        if let Some(x) = self.regs.get_mut(r as usize) {
            *x = Some(v);
        }
    }
    /// An op on the cards a register holds or a register stands for: possible when what filled it found cards.
    fn on_reg(&self, from: &ZoneRef, cards: &crate::spec::ops::cards::CardSel) -> Option<bool> {
        use crate::spec::ops::cards::CardSel;
        match (cards, from.1) {
            (CardSel::Chosen(r), _) => Some(self.reg(*r).unwrap_or(true)),
            (_, Zone::Scratch(r)) => Some(self.reg(r).unwrap_or(true)),
            _ => None,
        }
    }
}

impl RulesWalk {
    /// Would one effect of `steps` change the game (under `objects`: has every effect something to act on; `None`: no
    /// effect op)? Top-level costs and choices are gates.
    fn walk(&mut self, g: &Game, me: CardId, f: &Frame, steps: &[Step], top: bool) -> Option<bool> {
        let mut out = None;
        // One effect that would change the game is enough; under `objects` every effect needs something to act on.
        let all = self.objects;
        let join = |out: &mut Option<bool>, x: Option<bool>| match x {
            Some(true) if !all || out.is_none() => *out = Some(true),
            Some(false) if all || out.is_none() => *out = Some(false),
            _ => {}
        };
        for s in steps {
            // "If you use this Ability, ...": a consequence of using it, neither a gate nor an effect.
            if s.at == RuleStep::Consequence {
                continue;
            }
            // A choice a nested list (a coin's result, an "If") starts with that has nothing to choose: the rest of that
            // list can't happen (under `objects` its effects have nothing to act on).
            if !top && matches!(s.op, Op::Pick(_) | Op::PickSlot(_)) && !self.implied_ok(g, me, f, &s.op) {
                return if out == Some(true) && !self.objects { Some(true) } else { Some(false) };
            }
            if let Op::Pick(p) = &s.op {
                if p.from == ZoneRef(Who::Me, Zone::Hand) {
                    self.left_hand += crate::spec::value::num(g, me, f, &p.bounds.min).max(0);
                }
                let v = match p.from.1 {
                    Zone::Scratch(r) => self.reg(r).unwrap_or(true),
                    Zone::Deck if self.objects => self.implied_ok(g, me, f, &s.op),
                    _ => ops::pick_finds(g, me, f, p),
                };
                self.set_reg(p.into, v);
            }
            // The cards an op takes from a register: what filled it.
            let reg_fx = match &s.op {
                Op::Discard(DiscardSpec { from, cards, .. })
                | Op::PutIntoHand(PutIntoHandSpec { from, cards, .. })
                | Op::PutIntoDeck(PutIntoDeckSpec { from, cards, .. })
                | Op::Attach(AttachSpec { from, cards, onto: Some(_), .. }) => self.on_reg(from, cards),
                Op::Reveal(RevealSpec { cards: RevealWhat::Chosen(r), .. }) => Some(self.reg(*r).unwrap_or(true)),
                Op::PlayFromZone(pz) => Some(self.reg(pz.cards).unwrap_or(true) && ops::implied_ok(g, me, f, &s.op)),
                Op::Look(l) => {
                    let v = match self.on_reg(&l.from, &l.cards) {
                        Some(v) => v,
                        None => ops::implied_ok(g, me, f, &s.op),
                    };
                    self.set_reg(l.into, v);
                    Some(v)
                }
                _ => None,
            };
            if s.at == RuleStep::Cost {
                // "Discard your hand" can always be paid: a player always has a hand, even of zero cards (id2420).
                let whole = matches!(&s.op, Op::Discard(DiscardSpec { cards: CardSel::All, .. }) | Op::PutIntoDeck(PutIntoDeckSpec { cards: CardSel::All, .. }));
                self.gates_ok &= whole || ops::implied_ok(g, me, f, &s.op);
                continue;
            }
            // Under `objects` a condition read before the text does anything (flips a coin, chooses, moves a card) is read
            // as the board is now, unless it reads what a hidden zone holds.
            let now = self.now || (self.objects && !self.acted);
            if self.objects && !matches!(s.op, Op::If(_) | Op::Damage(_)) {
                self.acted = true;
            }
            let hidden = |c: &Cond| self.objects && crate::spec::value::cond_reads_hidden_cards(c);
            let holds_now = match &s.op {
                Op::If(i) if now && !ops::reads_done(&i.cond) && !hidden(&i.cond) => cond_now(g, me, f, &i.cond),
                _ => None,
            };
            let object = self.objects.then(|| ops::has_object(g, me, f, &s.op, now)).flatten();
            let fx = match &s.op {
                _ if reg_fx.is_some() => reg_fx,
                _ if object.is_some() => object,
                // Under `objects` the attack's damage is an effect when it is above 0 (`ops::has_object`).
                Op::Damage(_) if self.objects => None,
                Op::Draw(d) if self.left_hand > 0 => Some(ops::draw_possible_after(g, me, f, d, self.left_hand)),
                Op::If(i) if ops::reads_done(&i.cond) => None,
                Op::If(i) if holds_now.is_some() => {
                    let l = if holds_now == Some(true) { i.yes } else { i.no };
                    self.walk(g, me, f, l, false)
                }
                Op::If(i) if crate::spec::value::cond_stable(&i.cond) && !hidden(&i.cond) => {
                    let l = if crate::spec::value::cond(g, me, f, &i.cond) { i.yes } else { i.no };
                    self.walk(g, me, f, l, false)
                }
                Op::If(i) => {
                    let a = self.walk(g, me, f, i.yes, false);
                    let b = self.walk(g, me, f, i.no, false);
                    if a == Some(true) || b == Some(true) { Some(true) } else { a.or(b) }
                }
                Op::May(m) => {
                    let a = self.walk(g, me, f, m.yes, false);
                    let b = self.walk(g, me, f, m.no, false);
                    if a == Some(true) || b == Some(true) { Some(true) } else { a.or(b) }
                }
                // A coin flip counts through what its results do (id925; `flow::effect_possible`).
                Op::Coin(c) => {
                    let r = [self.walk(g, me, f, c.heads, false), self.walk(g, me, f, c.tails, false), self.walk(g, me, f, c.then, false)];
                    if r.contains(&Some(true)) { Some(true) } else { r.iter().copied().flatten().next() }
                }
                op => ops::effect_possible(g, me, f, op),
            };
            match fx {
                None => {
                    if top && !matches!(s.op, Op::If(_) | Op::May(_)) {
                        self.gates_ok &= self.implied_ok(g, me, f, &s.op);
                    }
                }
                x => join(&mut out, x),
            }
        }
        out
    }
}

/// The declared checks of a Trainer's effect (`reduce` and legality both make them): the card's `needs` and implied
/// preconditions (the turn's rules, one Supporter per turn, are the PlayTrainer's own checks, `engine::play_trainer`; a
/// used Supporter has none of them).
pub(crate) fn check_play<F: crate::derived::Facts>(g: &mut F, me: CardId, play: &PlaySpec, f: &Frame) -> R {
    // A Trainer whose effect is used by another card (Look-Alike Show) keeps its printed conditions only (user decision
    // D5; official JP FAQ, Fire Energy MEE 2 / Mela): it may do nothing.
    if f.via_attack {
        for c in play.needs.iter().filter(|c| matches!(c, Cond::Printed(_))) {
            if !cond_m(g, me, f, c)? {
                crate::bail!("CANNOT_PLAY_THIS_CARD");
            }
        }
        return Ok(());
    }
    // At a table that judges a Trainer from the known cards only, a condition on what a hidden zone holds isn't read
    // (`TableRules::trainer_conditions_on_hidden_cards_unread`); its printed conditions are.
    let hidden_unread = g.g().st.rules.trainer_conditions_on_hidden_cards_unread;
    for c in play.needs {
        if hidden_unread && !matches!(c, Cond::Printed(_)) && crate::spec::value::cond_reads_hidden_cards(c) {
            continue;
        }
        if !cond_m(g, me, f, c)? {
            crate::bail!("CANNOT_PLAY_THIS_CARD");
        }
    }
    if !usable_by_rules(g.g(), me, f, play.steps) {
        crate::bail!("CANNOT_PLAY_THIS_CARD");
    }
    Ok(())
}

/// Do the printed conditions of Trainer `me` hold for its Used view, its effect used by player `p`'s attack `atk`
/// (`Op::UseTrainer`)?
pub(crate) fn used_trainer_conditions(g: &mut Game, me: CardId, p: usize, atk: EffId) -> R<bool> {
    let Some(play) = crate::cards::spec_for(g.st.cards[me as usize].def).and_then(|s| s.play.as_ref()) else { return Ok(true) };
    let mut f = Frame::new(Prog::Play, Phase::Use, atk, p, frame_cause(g, me, Prog::Play, p as u8, atk, true));
    f.via_attack = true;
    for c in play.needs.iter().filter(|c| matches!(c, Cond::Printed(_))) {
        if !cond_m(g, me, &f, c)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The declared checks of using an Ability: once per turn, then its `needs` and implied preconditions.
pub(crate) fn check_power<F: crate::derived::Facts>(g: &mut F, me: CardId, pw: &PowerSpec, f: &Frame) -> R {
    check_once(g.g(), me, pw, f.p as usize)?;
    if !usable(g, me, f, pw.needs, pw.steps)? {
        crate::bail!("CANNOT_USE_POWER");
    }
    Ok(())
}

/// The usage record the use of the power `pw` of `me` (player `p`'s) makes, by its `once` (`None`: it isn't limited):
/// once during your turn (the card's, ending with the turn), the player's limit for every copy.
fn power_once(g: &Game, me: CardId, pw: &PowerSpec, p: usize) -> Option<(crate::state::UsageKey, crate::state::UsageBy, crate::state::Until)> {
    use crate::state::{Until, UsageBy};
    Some(match pw.once {
        Once::PerTurn => (crate::engine::usage::ability_key(&g.st, me, pw.index), UsageBy::Card(me), Until::ThisTurn(p as u8)),
        Once::PerTurnShared => (crate::engine::usage::ability_key(&g.st, me, pw.index), UsageBy::Player(p as u8), Until::ThisTurn(p as u8)),
        Once::No => return None,
    })
}

/// Once per turn: refused while the record is there.
fn check_once(g: &Game, me: CardId, pw: &PowerSpec, p: usize) -> R {
    if let Some((key, by, _)) = power_once(g, me, pw, p) {
        if crate::engine::usage::has(&g.st, key, by) {
            crate::bail!("POWER_ALREADY_USED");
        }
    }
    Ok(())
}

/// The declared checks of using the Stadium in play.
pub(crate) fn check_use_stadium<F: crate::derived::Facts>(g: &mut F, me: CardId, us: &PlaySpec, f: &Frame) -> R {
    if !usable(g, me, f, us.needs, us.steps)? {
        crate::bail!("CANNOT_USE_STADIUM");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Legality: the declared checks of a card, evaluated without running its program. `e` is the effect the
// use would carry (made by the caller on its scratch game); each returns `Ok` for a card that declares none.

/// A Trainer's play checks: player `p` plays `me` from the hand; `excl` the cards out of the hand while it resolves (a
/// played Item and its companion are in the play area).
pub fn trainer_play_check<F: crate::derived::Facts>(g: &mut F, me: CardId, p: usize, excl: [CardId; 2]) -> R {
    let Some(spec) = crate::cards::spec_for(g.g().st.cards[me as usize].def) else { return Ok(()) };
    let Some(play) = &spec.play else { return Ok(()) };
    let f = Frame::for_check(Prog::Play, Phase::Use, p, Cause::of_trainer(g.g(), me, p as u8), excl);
    check_play(g, me, play, &f)
}

/// Legality, the cheap half of the power checks (read from the game as it is): is the once-per-turn usage record free?
pub fn power_once_check(g: &Game, me: CardId, index: u8, p: usize) -> R {
    let Some(spec) = crate::cards::spec_for(g.st.cards[me as usize].def) else { return Ok(()) };
    match spec.powers.iter().find(|pw| pw.index == index) {
        Some(pw) => check_once(g, me, pw, p),
        None => Ok(()),
    }
}

/// The checks of using the power with printed index `index` of `me` (player `p`).
pub fn power_check<F: crate::derived::Facts>(g: &mut F, me: CardId, index: u8, p: usize) -> R {
    let Some(spec) = crate::cards::spec_for(g.g().st.cards[me as usize].def) else { return Ok(()) };
    for (i, pw) in spec.powers.iter().enumerate() {
        if pw.index == index {
            let f = Frame::for_check(Prog::Power(i as u8), Phase::Use, p, power_cause(g.g(), me, i as u8, p as u8), [NO_CARD; 2]);
            return check_power(g, me, pw, &f);
        }
    }
    Ok(())
}

/// The checks of using the Stadium `me` in play (player `p`).
pub fn use_stadium_check<F: crate::derived::Facts>(g: &mut F, me: CardId, p: usize) -> R {
    let Some(spec) = crate::cards::spec_for(g.g().st.cards[me as usize].def) else { return Ok(()) };
    let Some(us) = &spec.use_stadium else { return Ok(()) };
    let f = Frame::for_check(Prog::UseStadium, Phase::Use, p, Cause::new(CauseKind::Stadium, Some(me), p as u8), [NO_CARD; 2]);
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

/// The leading `Fail` steps of attack `attack`'s text before the damage (`p` the attacking player; `me` the card whose
/// program runs: the attack's card, or the copycat when the attack is copied from a Benched Pokémon), evaluated as `run`
/// does, in the frame the attack's program would have (its cause; no effect: the leading conditions read none).
pub fn attack_gate<F: crate::derived::Facts>(g: &mut F, me: CardId, attack: crate::state::AttackRef, p: usize) -> R<Gate> {
    let Some(spec) = crate::cards::spec_for(g.g().st.cards[attack.card as usize].def) else { return Ok(Gate::Open) };
    let Some((i, a)) = spec.attacks.iter().enumerate().find(|(_, a)| a.index == attack.idx() as u8) else { return Ok(Gate::Open) };
    let f = Frame::for_check(Prog::Attack(i as u8), Phase::BeforeDamage, p, Cause { kind: CauseKind::Attack, card: Some(me), player: p as u8, attack: Some(attack) }, [NO_CARD; 2]);
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

/// Could player `p`'s attack `attack`, used by the Pokémon in `attacking`, do anything: it does damage (its printed
/// damage), or the rules' walk of `usable_by_rules` over its text, asking whether each effect has something to act on
/// (`RulesWalk::objects`), finds every choice the text starts with has something to choose and an effect, each of the
/// effects the text does with something to act on (a text with no effect left, its conditions read as the board is
/// now, can't). Read only under the table rule
/// `TableRules::hide_attacks_that_can_do_nothing` (`engine::turn_action::attack_checks`). The text runs as the card that
/// prints the attack when it is in `attacking`, else as the Pokémon there (the copycat), in the frame the attack's
/// program would have (its cause; no effect yet).
pub(crate) fn attack_can_do_something(g: &Game, p: usize, attacking: crate::effects::SlotRef, attack: crate::state::AttackRef) -> bool {
    if crate::engine::attack::attack_def(g, attack).damage != 0 {
        return true;
    }
    let Some(spec) = crate::cards::spec_for(g.st.cards[attack.card as usize].def) else { return true };
    let Some((i, a)) = spec.attacks.iter().enumerate().find(|(_, a)| a.index == attack.idx() as u8) else { return true };
    let stack = g.st.slot(attacking.p as usize, attacking.s);
    let me = if stack.cards().contains(attack.card) || stack.tools().contains(attack.card) {
        attack.card
    } else {
        g.st.slot_pokemon(attacking.p as usize, attacking.s).unwrap_or(attack.card)
    };
    let f = Frame::for_check(Prog::Attack(i as u8), Phase::BeforeDamage, p, Cause { kind: CauseKind::Attack, card: Some(me), player: p as u8, attack: Some(attack) }, [NO_CARD; 2]);
    RulesWalk { objects: true, ..RulesWalk::new(false) }.usable(g, me, &f, a.steps)
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
                // The consequences of using the Ability are done (the Pokémon's Knock Out is pending): the rest runs; the
                // state check after the effect handles the Knock Out.
                Phase::Consequences => {
                    f.phase = Phase::AfterConsequences;
                    f.path = [0; MAX_DEPTH];
                    f.sub = 0;
                    continue;
                }
                _ => {}
            }
            // The program is done: the effects activated by the Pokémon it put into play apply now.
            return order::after_effect(g, me);
        }
        let step = &list[i];
        if f.depth == 0 && !runs_in(step.at, f.phase) {
            f.advance();
            continue;
        }
        // At a table that asks an attack's choices when their effect runs, step D makes only the choices the damage reads
        // (`TableRules::attack_choices_asked_when_their_effect_runs`); the others are asked after the damage.
        if f.depth == 0 && f.phase == Phase::Choices && g.st.rules.attack_choices_asked_when_their_effect_runs && !damage_reads(&list[i..]) {
            f.advance();
            continue;
        }
        // Every op step may write (records, flags, the lasting table): the derived facts are stale.
        g.st.bump();
        let flow = if f.phase == Phase::Choices { ops::choice(g, me, &mut f, &step.op)? } else { ops::exec(g, me, &mut f, &step.op)? };
        g.st.bump();
        match flow {
            Flow::Next => f.advance(),
            Flow::Enter(sel) => f.enter(sel),
            Flow::Suspend => return Ok(()),
        }
    }
}

#[cfg(test)]
mod usability_tests {
    //! The rules' usability (`usable_by_rules`, user decision D10) against the rulings' cases.
    use super::*;
    use serde_json::json;

    fn rules(sc: serde_json::Value, card: &str) -> bool {
        let mut names: Vec<&str> = Vec::new();
        for n in [
            "Ultra Ball 30C 128",
            "Lillie's Determination ASC 192",
            "Carmine PRE 103",
            "Special Red Card CRI 82",
            "Hop's Snorlax JTG 117",
            "Iris's Fighting Spirit ASC 190",
            "Kofu SCR 138",
            "Judge POR 76",
            "Energy Swatter POR 73",
            "Eri PRE 136",
            "Crushing Hammer PBL 105",
        ] {
            names.extend(std::iter::repeat(n).take(4));
        }
        while names.len() < 60 {
            names.push("Psychic Energy MEE 5");
        }
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &sc).unwrap();
        let p = g.st.active_player as usize;
        let def = crate::carddb::def_by_full_name(card).unwrap();
        let me = g.st.players[p].hand().iter().find(|c| g.st.cards[*c as usize].def == def).expect("the card in hand");
        let play = crate::cards::spec_for(def).unwrap().play.as_ref().unwrap();
        let f = Frame::start(&g, me, Prog::Play, Phase::Use, 0, p, false);
        usable_by_rules(&g, me, &f, play.steps)
    }

    /// id1188: Ultra Ball with an empty deck can't be played (the discard is a cost, not the effect).
    #[test]
    fn ultra_ball_empty_deck() {
        let base = |n: u64| json!({"me": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": ["Ultra Ball 30C 128", "Psychic Energy MEE 5", "Psychic Energy MEE 5"], "deck_left": n}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117"}});
        assert!(!rules(base(0), "Ultra Ball 30C 128"));
        assert!(rules(base(10), "Ultra Ball 30C 128"));
    }

    /// id1124 (Professor Juniper; Carmine's "Discard your hand and draw 5 cards" alike): with an empty deck only the cost is
    /// left; with a deck, an empty hand still pays it (id2420: a player always has a hand).
    #[test]
    fn carmine_discard_is_a_cost() {
        let sc = |n: u64, hand: Vec<&str>| json!({"me": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": hand, "deck_left": n}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117"}});
        assert!(!rules(sc(0, vec!["Carmine PRE 103", "Psychic Energy MEE 5"]), "Carmine PRE 103"));
        assert!(rules(sc(10, vec!["Carmine PRE 103"]), "Carmine PRE 103"));
    }

    /// id2365: Special Red Card with an empty opponent's hand can't be played.
    #[test]
    fn special_red_card_empty_opponent_hand() {
        let sc = |hand: Vec<&str>| json!({"me": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": ["Special Red Card CRI 82"]}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": hand}});
        assert!(!rules(sc(vec![]), "Special Red Card CRI 82"));
        assert!(rules(sc(vec!["Psychic Energy MEE 5"]), "Special Red Card CRI 82"));
    }

    /// id956 (Captivating Poke Puff: "You cannot play a Trainer card for no net effect"): Energy Swatter with no cards in the
    /// opponent's hand can't be played; with any card it can (revealing the hand is an effect).
    #[test]
    fn energy_swatter_empty_opponent_hand() {
        let sc = |hand: Vec<&str>| json!({"me": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": ["Energy Swatter POR 73"]}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": hand}});
        assert!(!rules(sc(vec![]), "Energy Swatter POR 73"));
        assert!(rules(sc(vec!["Psychic Energy MEE 5"]), "Energy Swatter POR 73"));
    }

    /// id1492 (APR E-20, the first half): "Your opponent reveals their hand" with no cards in it isn't revealing it, so Eri
    /// can't be played (Drone Rotom: "you can't play Drone Rotom in the first place").
    #[test]
    fn eri_reveal_of_an_empty_hand() {
        let sc = |hand: Vec<&str>| json!({"me": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": ["Eri PRE 136"]}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": hand}});
        assert!(!rules(sc(vec![]), "Eri PRE 136"));
        assert!(rules(sc(vec!["Ultra Ball 30C 128"]), "Eri PRE 136"));
    }

    /// id1042 (Hugh: "draw until" that changes nothing can't be played): Iris's Fighting Spirit discards another card, then
    /// draws until 6. With 7 other cards the hand is 6 after the cost: nothing to draw. With 6 others it draws 1.
    #[test]
    fn iris_draw_until_that_changes_nothing() {
        let sc = |n: usize| {
            let mut hand = vec!["Iris's Fighting Spirit ASC 190"];
            hand.extend(std::iter::repeat("Psychic Energy MEE 5").take(n));
            json!({"me": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": hand}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117"}})
        };
        assert!(!rules(sc(7), "Iris's Fighting Spirit ASC 190"));
        assert!(rules(sc(6), "Iris's Fighting Spirit ASC 190"));
    }

    /// id1063 (Lost Blender: a cost of 2 needs 2 cards): Kofu puts 2 cards from the hand on the bottom of the deck; with 1
    /// other card it can't be played.
    #[test]
    fn kofu_needs_two_cards() {
        let sc = |n: usize| {
            let mut hand = vec!["Kofu SCR 138"];
            hand.extend(std::iter::repeat("Psychic Energy MEE 5").take(n));
            json!({"me": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": hand}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117"}})
        };
        assert!(!rules(sc(1), "Kofu SCR 138"));
        assert!(rules(sc(2), "Kofu SCR 138"));
    }

    /// id966 (Colress with zero cards in both hands and no Benched Pokémon: "Yes, you can"): Judge with no other card in
    /// either hand still shuffles the "hand of zero cards" and draws, so it can be played.
    #[test]
    fn judge_with_zero_hands() {
        let sc = json!({"me": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": ["Judge POR 76"]}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117"}});
        assert!(rules(sc, "Judge POR 76"));
    }

    /// id925 ("you cannot play a card if it is obvious that you cannot do what the card is supposed to do"): Crushing Hammer's
    /// coin flip alone isn't what it does; with no Energy on the opponent's Pokémon it can't be played.
    #[test]
    fn crushing_hammer_no_energy() {
        let sc = |energy: Vec<&str>| json!({"me": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": ["Crushing Hammer PBL 105"]}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117", "active_energy": energy}});
        assert!(!rules(sc(vec![]), "Crushing Hammer PBL 105"));
        assert!(rules(sc(vec!["Psychic Energy MEE 5"]), "Crushing Hammer PBL 105"));
    }

    /// E-36 specific case, id2420: Lillie's Determination as the only card in the hand is playable.
    #[test]
    fn lillies_determination_only_card() {
        let sc = json!({"me": {"reset": true, "active": "Hop's Snorlax JTG 117", "hand": ["Lillie's Determination ASC 192"]}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117"}});
        assert!(rules(sc, "Lillie's Determination ASC 192"));
    }
}

#[cfg(test)]
mod consequence_tests {
    //! "If you use this Ability, this Pokémon is Knocked Out" (`Step::consequence`): never counted toward usability, always
    //! carried out.
    use super::*;
    use serde_json::json;

    /// The rules' usability of the first Ability of the Pokémon `card` in play for the turn player, on the board `sc` (decks:
    /// 4 of each name, Lightning Energy to 60).
    fn power_rules(names: &[&str], sc: serde_json::Value, card: &str) -> bool {
        let mut deck: Vec<&str> = Vec::new();
        for n in names {
            deck.extend(std::iter::repeat(*n).take(4));
        }
        while deck.len() < 60 {
            deck.push("Lightning Energy MEE 4");
        }
        let deck: Vec<u16> = deck.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &sc).unwrap();
        let p = g.st.active_player as usize;
        let def = crate::carddb::def_by_full_name(card).unwrap();
        let me = g.st.players[p].slots.iter().flat_map(|s| s.cards().iter()).find(|c| g.st.cards[*c as usize].def == def).expect("the Pokémon in play");
        let pw = &crate::cards::spec_for(def).unwrap().powers[0];
        let f = Frame::start(&g, me, Prog::Power(0), Phase::Use, 0, p, false);
        usable_by_rules(&g, me, &f, pw.steps)
    }

    /// Official JP FAQ (related: レアコイル): 「トラッシュに基本エネルギーが1枚もないとき、レアコイルの特性「かじょうほうでん」を使う
    /// ことはできますか？」「いいえ、できません。」 = with no Basic Energy in the discard pile, Overvolt Discharge can't be used:
    /// its Knock Out is a consequence of using it, not an effect that makes it usable.
    #[test]
    fn magneton_overvolt_discharge_needs_basic_energy_in_the_discard_pile() {
        let names = ["Magneton SSP 59", "Hop's Snorlax JTG 117"];
        let sc = |discard: Vec<&str>| json!({"me": {"reset": true, "active": "Magneton SSP 59", "discard": discard}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117"}});
        assert!(!power_rules(&names, sc(vec![]), "Magneton SSP 59"));
        assert!(!power_rules(&names, sc(vec!["Hop's Snorlax JTG 117"]), "Magneton SSP 59"));
        assert!(power_rules(&names, sc(vec!["Lightning Energy MEE 4"]), "Magneton SSP 59"));
    }

    /// Dusknoir / Dusclops PRE 37 / 36 Cursed Blast: usable whenever the counters can be put on an opponent's Pokémon; the
    /// Knock Out is not what makes it usable.
    #[test]
    fn cursed_blast_usable_by_its_counters() {
        for card in ["Dusknoir PRE 37", "Dusclops PRE 36"] {
            let sc = json!({"me": {"reset": true, "active": card}, "opp": {"reset": true, "active": "Hop's Snorlax JTG 117"}});
            assert!(power_rules(&[card, "Hop's Snorlax JTG 117"], sc, card));
        }
    }

    /// Psyduck ASC 39's Damp removes "any Ability that requires the Pokémon using it to Knock Out itself": exactly the
    /// Abilities whose program has a consequence Knocking this Pokémon Out (what the card data's `knocks_out_self` marked,
    /// proved equal on every power of every spec before the flag left the data): the pool's are Cursed Blast and Overvolt
    /// Discharge.
    #[test]
    fn damp_names_the_consequence_knock_outs() {
        let mut users: Vec<&str> = Vec::new();
        for (i, d) in crate::carddb::cards().iter().enumerate() {
            let Some(spec) = crate::cards::spec_for(i as u16) else { continue };
            for pw in spec.powers {
                if program_knocks_out_self(pw) {
                    users.push(d.full_name);
                }
            }
        }
        for n in ["Dusclops PRE 36", "Dusknoir PRE 37", "Magneton SSP 59"] {
            assert!(users.contains(&n), "{n}");
        }
    }
}

#[cfg(test)]
mod deferred_tests {
    //! An attack's removal of attached cards waits for its damage (`knockout::leave_play_cards` with a window) and reports
    //! `Done` when queued: no attack may read "if you do" after one, or it would read the queued answer.
    use super::*;

    fn defers(op: &Op) -> bool {
        matches!(op, Op::DiscardEnergy(_)) || matches!(op, Op::Discard(DiscardSpec { from: ZoneRef(_, Zone::Attached(_) | Zone::AttachedEnergy(_) | Zone::Tools(_)), .. }))
    }

    fn reads_done_after(steps: &[Step], seen: &mut bool) -> bool {
        for s in steps {
            if *seen && matches!(&s.op, Op::If(i) if ops::reads_done(&i.cond)) {
                return true;
            }
            if defers(&s.op) {
                *seen = true;
            }
            for sel in 0..=8 {
                if reads_done_after(ops::child(&s.op, sel), seen) {
                    return true;
                }
            }
        }
        false
    }

    #[test]
    fn no_if_you_do_after_a_deferred_removal() {
        let mut bad = Vec::new();
        for (i, d) in crate::carddb::cards().iter().enumerate() {
            let Some(spec) = crate::cards::spec_for(i as u16) else { continue };
            for a in spec.attacks {
                if reads_done_after(a.steps, &mut false) {
                    bad.push(format!("{} attack {}", d.full_name, a.index));
                }
            }
        }
        bad.dedup();
        assert!(bad.is_empty(), "an attack reads \"if you do\" after a removal deferred to after its damage: {:?}", bad);
    }
}

#[cfg(test)]
mod outcome_tests {
    use super::*;

    /// The outcome of an op that moves cards (events design section 7), and its round trip through the card frame.
    #[test]
    fn outcomes() {
        assert_eq!(Outcome::count(0, 0, 0), Outcome::Impossible);
        assert_eq!(Outcome::count(2, 2, 0), Outcome::Done);
        assert_eq!(Outcome::count(2, 1, 1), Outcome::Partial);
        assert_eq!(Outcome::count(2, 0, 2), Outcome::Prevented);
        assert_eq!(Outcome::count(2, 0, 0), Outcome::Impossible);
        assert!(Outcome::Done.did() && Outcome::Partial.did() && !Outcome::Prevented.did() && !Outcome::Impossible.did());
        let g = crate::game::Game::new(1);
        for o in [Outcome::Done, Outcome::Partial, Outcome::Prevented, Outcome::Impossible] {
            let mut f = Frame::new(Prog::Play, Phase::Use, 0, 1, Cause::rule(crate::cause::RuleWhich::Action, 1));
            f.outcome = o;
            f.via_attack = true;
            f.attached_to = 0x13;
            let back = Frame::decode(&g, 0, &f.frame_at(5)).unwrap();
            assert_eq!(back.outcome, o);
            assert!(back.via_attack && back.attached_to == 0x13 && back.p == 1);
        }
    }
}

/// Does the attack's damage read the choice of the first of `rest` (a top-level step of the attack's program and the
/// steps after it)? The damage reads a choice through `Op::ChoiceDamage` ("N damage for each card you discarded in this
/// way", "to have this attack do N more damage", "if you can't discard N cards in this way, this attack does nothing"),
/// which counts what was chosen before it: the step holding it and every step before it.
fn damage_reads(rest: &[Step]) -> bool {
    fn holds(steps: &[Step]) -> bool {
        steps.iter().any(|s| matches!(s.op, Op::ChoiceDamage(_)) || (0..=8).any(|sel| holds(ops::child(&s.op, sel))))
    }
    rest.iter().any(|s| s.at == RuleStep::AfterDamage && holds(std::slice::from_ref(s)))
}

fn runs_in(at: RuleStep, phase: Phase) -> bool {
    matches!(
        (at, phase),
        (RuleStep::BeforeDamage, Phase::BeforeDamage)
            | (RuleStep::AfterDamage, Phase::AfterDamage)
            | (RuleStep::AfterDamage, Phase::Choices)
            | (RuleStep::Use, Phase::Use)
            | (RuleStep::Cost, Phase::Use)
            | (RuleStep::Consequence, Phase::Consequences)
            | (RuleStep::Cost, Phase::AfterConsequences)
            | (RuleStep::Use, Phase::AfterConsequences)
    )
}

#[cfg(test)]
mod power_cause_tests {
    //! A power's cause follows its printed kind (`PowerDef::source`): an Ability's is `Ability`, a Fossil's "you may discard
    //! this card from play" (a rule printed on the card) is `Rule { CardRule }`.
    use super::*;
    use crate::cause::{CauseKind, RuleWhich};

    #[test]
    fn a_fossils_discard_is_a_card_rule_not_an_ability() {
        let kind = |card: &str, power: u8| {
            let def = crate::carddb::def_by_full_name(card).unwrap();
            let deck: Vec<u16> = std::iter::repeat(def).take(60).collect();
            let mut g = Game::new(7);
            g.start([&deck, &deck]).unwrap();
            let me = g.st.cards.iter().position(|c| c.def == def).unwrap() as CardId;
            frame_cause(&g, me, Prog::Power(power), 0, 0, false).kind
        };
        assert_eq!(kind("Antique Root Fossil SCR 130", 0), CauseKind::Rule { which: RuleWhich::CardRule });
        assert_eq!(kind("Antique Cover Fossil SCR 129", 0), CauseKind::Rule { which: RuleWhich::CardRule });
        assert_eq!(kind("Mandibuzz WHT 64", 0), CauseKind::Ability);
    }
}

#[cfg(test)]
mod hidden_needs_tests {
    use super::*;

    /// The Trainers whose `needs` read what a hidden zone holds (`value::cond_reads_hidden_cards`), not as a printed
    /// condition: the only ones `TableRules::trainer_conditions_on_hidden_cards_unread` offers where the rulings don't (a
    /// named deck search in a Trainer's program names only Basic Energy, which no copy limit stops, so the usability walk
    /// reads nothing hidden for a Trainer).
    #[test]
    fn trainers_with_needs_on_hidden_cards() {
        let mut names: Vec<&str> = Vec::new();
        for (i, d) in crate::carddb::cards().iter().enumerate() {
            let Some(play) = crate::cards::spec_for(i as u16).and_then(|s| s.play.as_ref()) else { continue };
            if play.needs.iter().any(|c| !matches!(c, Cond::Printed(_)) && crate::spec::value::cond_reads_hidden_cards(c)) {
                names.push(d.name);
            }
        }
        names.sort();
        names.dedup();
        assert_eq!(names, vec!["Love Ball", "Salvatore"]);
    }
}
