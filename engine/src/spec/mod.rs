//! Declarative card specs (PLAN.md 8.5): a card with logic is a `static
//! CardSpec` run by one shared interpreter (`run.rs`) instead of its own
//! `reduce`/`resume` handlers.
//!
//! The vocabulary is `porting/card-survey/vocabulary-v1.md` (approved
//! 2026-10-08); names here follow it. Only the items converted cards use are
//! implemented so far; each new item is added here and in `run.rs`, never in a
//! card file.
//!
//! A converted card file defines `pub static SPEC: CardSpec` and
//! `pub static IMPL: CardImpl = SPEC.card_impl();`. `build.rs` registers both;
//! converted and hand-written cards run side by side.
//!
//! Parity is on what a player can observe (PLAN.md 8.5): ops implement the
//! rules, not Twinleaf's internal plumbing.

pub mod run;

use crate::cards::CardImpl;
use crate::effects::{k, KindMask};

pub struct CardSpec {
    /// Twinleaf behavior class (registry key, as `CardImpl::class`).
    pub class: &'static str,
    pub attacks: &'static [AttackSpec],
    pub powers: &'static [PowerSpec],
    pub play: Option<PlaySpec>,
    pub passives: &'static [Passive],
}

impl CardSpec {
    pub const NONE: CardSpec = CardSpec { class: "", attacks: &[], powers: &[], play: None, passives: &[] };

    /// The registry entry: the shared interpreter, subscribed to exactly the
    /// effect kinds this spec reacts to.
    pub const fn card_impl(&'static self) -> CardImpl {
        CardImpl { class: self.class, mask: self.mask(), reduce: run::reduce, resume: Some(run::resume), coin: None, can_play: None }
    }

    const fn mask(&self) -> KindMask {
        let mut m = KindMask::EMPTY;
        let mut i = 0;
        while i < self.attacks.len() {
            let steps = self.attacks[i].steps;
            let mut j = 0;
            while j < steps.len() {
                m = with(m, match steps[j].at {
                    RuleStep::BeforeDamage => k::ATTACK,
                    RuleStep::AfterDamage => k::AFTER_ATTACK,
                    RuleStep::Use => k::ATTACK,
                });
                j += 1;
            }
            i += 1;
        }
        if !self.powers.is_empty() {
            m = with(m, k::POWER);
        }
        if self.play.is_some() {
            m = with(m, k::TRAINER);
        }
        let mut i = 0;
        while i < self.passives.len() {
            m = with(m, match self.passives[i].modifier {
                Modifier::HpBonus(_) => k::CHECK_HP,
            });
            i += 1;
        }
        m
    }
}

const fn with(mut m: KindMask, kind: u32) -> KindMask {
    m.0[(kind >> 6) as usize] |= 1u64 << (kind & 63);
    m
}

/// An attack's text: its steps, each placed at a rulebook step of the attack.
pub struct AttackSpec {
    /// Index of the attack on the card.
    pub index: u8,
    pub steps: &'static [Step],
}

/// A Trainer's effect when played.
pub struct PlaySpec {
    pub kind: PlayKind,
    /// Preconditions beyond those its ops imply; the card can't be played
    /// when one fails.
    pub needs: &'static [Cond],
    pub steps: &'static [Step],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayKind {
    Item,
    Supporter,
    Tool,
    Stadium,
}

/// An activated Ability.
pub struct PowerSpec {
    /// Index of the power on the card.
    pub index: u8,
    pub once: Once,
    /// Preconditions beyond those its ops imply.
    pub needs: &'static [Cond],
    pub steps: &'static [Step],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Once {
    No,
}

pub struct Step {
    pub at: RuleStep,
    pub op: Op,
}

impl Step {
    /// A step of a Trainer, an Ability or a nested list (its position comes
    /// from its parent).
    pub const fn new(op: Op) -> Step {
        Step { at: RuleStep::Use, op }
    }
    pub const fn before_damage(op: Op) -> Step {
        Step { at: RuleStep::BeforeDamage, op }
    }
    pub const fn after_damage(op: Op) -> Step {
        Step { at: RuleStep::AfterDamage, op }
    }
}

/// Where a step runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleStep {
    /// Attack, before the damage.
    BeforeDamage,
    /// Attack, the attack's effects after the damage.
    AfterDamage,
    /// Trainer, Ability, or a step inside a nested list.
    Use,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Who {
    Me,
    Opp,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Zone {
    Deck,
    Hand,
    Discard,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ZoneRef(pub Who, pub Zone);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotExpr {
    /// The Pokémon this card is (or is attached to).
    This,
    Active(Who),
}

pub enum Op {
    Draw(DrawSpec),
    May(MaySpec),
    If(IfSpec),
    Search(SearchSpec),
    Shuffle(ShuffleSpec),
    DiscardEnergy(DiscardEnergySpec),
    HandShuffleDraw(HandShuffleDrawSpec),
    RemoveFromPlay(RemoveFromPlaySpec),
}

pub struct DrawSpec {
    pub who: Who,
    pub amount: DrawAmount,
}

pub enum DrawAmount {
    Count(Num),
    /// Draw until the hand has this many cards (no draw if it has as many).
    UntilHandSize(Num),
}

/// "You may": ask `asker` when `when` holds (otherwise nothing is asked);
/// on yes run `yes`, on no run `no`.
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

/// Search a zone for cards matching `pick` and put them somewhere. An attack
/// whose search can't be carried out still resolves (rulings 336, 337, 1790).
pub struct SearchSpec {
    pub pick: PickSpec,
    pub destination: SearchDestination,
    pub msg: &'static str,
}

pub struct PickSpec {
    pub chooser: Who,
    pub from: ZoneRef,
    pub predicate: Pred,
    pub bounds: Bounds,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SearchDestination {
    /// Onto the chooser's Bench (as played from the zone), at most the open
    /// Bench spaces.
    Bench,
}

pub struct Bounds {
    pub min: Num,
    pub max: Num,
}

pub struct ShuffleSpec {
    pub zone: ZoneRef,
}

pub struct DiscardEnergySpec {
    pub target: SlotExpr,
    pub selection: EnergySelection,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnergySelection {
    /// Every card providing Energy to the Pokémon.
    AllProvided,
}

/// Shuffle a hand into its deck, then draw (the resolving card is not part
/// of the hand).
pub struct HandShuffleDrawSpec {
    pub who: Who,
    pub draw: Num,
}

/// Put a Pokémon and all cards attached to it into a zone.
pub struct RemoveFromPlaySpec {
    pub slot: SlotExpr,
    pub destination: ZoneRef,
}

pub enum Num {
    Lit(i32),
    ZoneSize(ZoneRef),
    OpenBench(Who),
    PrizesLeft(Who),
    Min(&'static Num, &'static Num),
    /// `if cond { a } else { b }`.
    If(&'static Cond, &'static Num, &'static Num),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CmpOp {
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
}

pub enum Cond {
    True,
    Not(&'static Cond),
    All(&'static [Cond]),
    Cmp(Num, CmpOp, Num),
    /// The zone holds a card matching the predicate.
    Nonempty(ZoneRef, Pred),
    BenchSpace(Who),
}

/// A card predicate.
pub enum Pred {
    Any,
    All(&'static [Pred]),
    Pokemon,
    Basic,
    HpAtMost(i32),
}

/// A modifier that applies while its card is in place.
pub struct Passive {
    pub origin: RuleSource,
    pub modifier: Modifier,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleSource {
    /// A Pokémon Tool: applies to the Pokémon it is attached to, unless Tools
    /// have no effect.
    Tool,
}

pub enum Modifier {
    /// +HP to the Pokémon.
    HpBonus(i32),
}

/// An attack choice made at step D (before the damage) and carried out after
/// it: the step it belongs to and the answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecChoice {
    pub card: crate::list::CardId,
    /// Program and path of the step (`run.rs`).
    pub key: u64,
    pub answer: u8,
}

/// Everything a spec card file needs.
pub mod prelude {
    pub use super::*;
    pub use crate::cards::CardImpl;
}
