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
}

impl Frame {
    pub(crate) fn new(prog: Prog, phase: Phase, eff: EffId, p: usize, cause: Cause) -> Frame {
        Frame { prog, phase, path: [0; MAX_DEPTH], depth: 0, iter: [0; MAX_DEPTH], sub: 0, eff, p: p as u8, cards: [NONE; 2], heads: 0, slot: NONE, prize: NONE, attached_to: NONE, last: 0, via_attack: false, outcome: Outcome::Impossible, cause }
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

/// The `Event::On` triggers over one view of the event `e` (the view of `card` of an event carrying cards), in the
/// propagation order of the declaring cards. The view is read for each trigger, as the board is when it is asked.
fn fire_on(g: &mut Game, e: EffId, kind: u32, card: Option<CardId>) -> R {
    let order = g.listeners(crate::game::prop_class(g.e(e)), kind);
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
                let mut f = Frame::start(g, c, Prog::Trigger(i as u8), Phase::Use, e, p, false);
                f.slot = slot;
                run(g, c, f)?;
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
    let old = steps.iter().filter(|s| s.at != RuleStep::Consequence).all(|s| ops::implied_ok(g, me, f, &s.op));
    // User decision D10, step 1: the rules' usability is computed beside today's under VERIFY and every difference is
    // recorded (`usability_report`); today's answer stands until the switch is approved.
    if crate::cause::on(g) {
        let new = usable_by_rules(g, me, f, steps);
        if new != old {
            record_usability_difference(g, me, f, old, new);
        }
    }
    Ok(old)
}

/// The rules' usability of a Trainer's or an Ability's program (events design section 7; APR A-02, B-01, D-02, E-20, E-30;
/// id925, id956: "you cannot play a Trainer card for no net effect"), the card's `needs` aside (its printed conditions,
/// checked first): every cost can be paid (`Step::cost`), every choice the program starts with has something to choose
/// (a Pick, a PickSlot: the program's gates), and at least one of its effects would change the game (an op under
/// `If(Done)` counts only through the op it depends on: E-20's "If you do"). A program with no effect op is usable.
pub(crate) fn usable_by_rules(g: &Game, me: CardId, f: &Frame, steps: &[Step]) -> bool {
    let mut w = RulesWalk { left_hand: 0, gates_ok: true, regs: [None; 4] };
    let fx = w.walk(g, me, f, steps, true);
    w.gates_ok && fx != Some(false)
}

/// The walk of `usable_by_rules`: the cards the program takes from its player's hand so far (they leave it: "draw until
/// you have 6 cards" counts after the discard), and whether every top-level cost and choice can be made.
struct RulesWalk {
    left_hand: i32,
    gates_ok: bool,
    /// Per card register: whether the op that fills it would find cards (`None`: not filled by the program yet).
    regs: [Option<bool>; 4],
}

impl RulesWalk {
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
    /// Would one effect of `steps` change the game (`None`: no effect op)? Top-level costs and choices are gates.
    fn walk(&mut self, g: &Game, me: CardId, f: &Frame, steps: &[Step], top: bool) -> Option<bool> {
        let mut out = None;
        let join = |out: &mut Option<bool>, x: Option<bool>| match x {
            Some(true) => *out = Some(true),
            Some(false) if out.is_none() => *out = Some(false),
            _ => {}
        };
        for s in steps {
            // "If you use this Ability, ...": a consequence of using it, neither a gate nor an effect.
            if s.at == RuleStep::Consequence {
                continue;
            }
            if let Op::Pick(p) = &s.op {
                if p.from == ZoneRef(Who::Me, Zone::Hand) {
                    self.left_hand += crate::spec::value::num(g, me, f, &p.bounds.min).max(0);
                }
                let v = match p.from.1 {
                    Zone::Scratch(r) => self.reg(r).unwrap_or(true),
                    _ => ops::implied_ok(g, me, f, &s.op),
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
            let fx = match &s.op {
                _ if reg_fx.is_some() => reg_fx,
                Op::Draw(d) if self.left_hand > 0 => Some(ops::draw_possible_after(g, me, f, d, self.left_hand)),
                Op::If(i) if ops::reads_done(&i.cond) => None,
                Op::If(i) if crate::spec::value::cond_stable(&i.cond) => {
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
                op => ops::effect_possible(g, me, f, op),
            };
            match fx {
                None => {
                    if top && !matches!(s.op, Op::If(_) | Op::May(_)) {
                        self.gates_ok &= ops::implied_ok(g, me, f, &s.op);
                    }
                }
                x => join(&mut out, x),
            }
        }
        out
    }
}

/// The differences between today's usability and the rules' one met under VERIFY, by card and answers, each with a count
/// and up to three situations (`usability_report`).
static USABILITY: std::sync::Mutex<std::collections::BTreeMap<String, (u64, Vec<String>)>> = std::sync::Mutex::new(std::collections::BTreeMap::new());

#[cold]
fn record_usability_difference(g: &Game, me: CardId, f: &Frame, old: bool, new: bool) {
    let p = f.p as usize;
    let pl = &g.st.players[p];
    let o = &g.st.players[1 - p];
    let key = format!("{} ({}): today {}, rules {}", g.st.cdef(me).full_name, match f.prog { Prog::Play => "play", Prog::Power(_) => "Ability", _ => "use" }, old, new);
    let situation = format!(
        "turn {} deck {} hand {} (without it) discard {} bench {} | opp hand {} deck {} bench {}",
        g.st.turn,
        pl.deck.len(),
        pl.hand.iter().filter(|c| *c != me).count(),
        pl.discard.len(),
        pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards.is_empty()).count(),
        o.hand.len(),
        o.deck.len(),
        o.bench.iter().filter(|b| !o.slots[**b as usize].cards.is_empty()).count()
    );
    let mut m = USABILITY.lock().unwrap();
    let e = m.entry(key).or_insert((0, Vec::new()));
    e.0 += 1;
    if e.1.len() < 3 && !e.1.contains(&situation) {
        e.1.push(situation);
    }
}

/// The usability differences of a VERIFY run (empty when there were none).
pub fn usability_report() -> String {
    let m = USABILITY.lock().unwrap();
    if m.is_empty() {
        return String::new();
    }
    let mut s = format!("usability differences (D10): {} cards\n", m.len());
    for (k, (n, sits)) in m.iter() {
        s.push_str(&format!("{:>8}  {}\n", n, k));
        for x in sits {
            s.push_str(&format!("          {}\n", x));
        }
    }
    s
}

/// The declared checks of a Trainer's effect (`reduce` and legality both make them): the card's `needs` and implied
/// preconditions (the turn's rules, one Supporter per turn, are the PlayTrainer's own checks, `engine::play_trainer`; a
/// used Supporter has none of them).
pub(crate) fn check_play(g: &mut Game, me: CardId, play: &PlaySpec, f: &Frame) -> R {
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
    if !usable(g, me, f, play.needs, play.steps)? {
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

#[cfg(test)]
mod usability_tests {
    //! The rules' usability (`usable_by_rules`, user decision D10) against the rulings' cases.
    use super::*;
    use serde_json::json;

    fn rules(sc: serde_json::Value, card: &str) -> bool {
        let mut names: Vec<&str> = Vec::new();
        for n in ["Ultra Ball 30C 128", "Lillie's Determination ASC 192", "Carmine PRE 103", "Special Red Card CRI 82", "Hop's Snorlax JTG 117"] {
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
        let me = g.st.players[p].hand.iter().find(|c| g.st.cards[*c as usize].def == def).expect("the card in hand");
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
        let me = g.st.players[p].slots.iter().flat_map(|s| s.cards.iter()).find(|c| g.st.cards[*c as usize].def == def).expect("the Pokémon in play");
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
    /// Abilities whose program has a consequence Knocking this Pokémon Out (the card data's `knocks_out_self`).
    #[test]
    fn damp_names_the_consequence_knock_outs() {
        for (i, d) in crate::carddb::cards().iter().enumerate() {
            let Some(spec) = crate::cards::spec_for(i as u16) else { continue };
            for pw in spec.powers {
                let ko = pw.steps.iter().any(|s| s.at == RuleStep::Consequence && matches!(&s.op, Op::KnockOut(k) if k.target == SlotExpr::This));
                let flagged = d.powers.get(pw.index as usize).map_or(false, |x| x.knocks_out_self);
                assert_eq!(ko, flagged, "{}: consequence Knock Out {} vs knocks_out_self {}", d.full_name, ko, flagged);
            }
        }
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

fn runs_in(at: RuleStep, phase: Phase) -> bool {
    matches!(
        (at, phase),
        (RuleStep::BeforeDamage, Phase::BeforeDamage)
            | (RuleStep::AfterDamage, Phase::AfterDamage)
            | (RuleStep::AfterDamage, Phase::Choices)
            | (RuleStep::Use, Phase::Use)
            | (RuleStep::Cost, Phase::Use)
            | (RuleStep::Consequence, Phase::Use)
    )
}
