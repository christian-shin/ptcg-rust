//! Declarative card specs: a card with logic is a `static
//! CardSpec` run by one shared interpreter (`run.rs`) instead of its own
//! `reduce`/`resume` handlers.
//!
//! The vocabulary is described in docs/ENGINE.md, "Cards as specs" (local);
//! the rules-events migration (docs/design/events-design.md) is replacing
//! parts of it. Every op is declared here; its record type,
//! executor and step-D choice live in its family file, which one person owns
//! at a time:
//!
//! | File | Items |
//! | --- | --- |
//! | `ops/cards.rs` | moving cards between zones, Energy, Prizes |
//! | `ops/board.rs` | damage, counters, healing, switching, conditions, Knock Outs, evolution |
//! | `ops/flow.rs` | May, If, Coin, Choose, loops, Fail, attack copying, ending the turn or game |
//! | `ops/state.rs` | lasting effects and markers |
//! | `value.rs` | conditions, numbers, card and slot predicates, selectors |
//! | `passive.rs` | passive modifiers |
//! | `trigger.rs` | triggers |
//!
//! A converted card file defines `pub static SPEC: CardSpec` and
//! `pub static IMPL: CardImpl = SPEC.card_impl();`. `build.rs` registers both;
//! converted and hand-written cards run side by side.
//!
//! Parity is on what a player can observe: ops implement the
//! rules, not Twinleaf's internal plumbing. An item that is declared but not
//! implemented yet panics when a spec reaches it, so a replay reports it.

pub mod ops;
pub mod passive;
pub mod run;
pub mod trigger;
pub mod value;

pub use ops::board::*;
pub use ops::cards::*;
pub use ops::flow::*;
pub use ops::state::*;
pub use passive::*;
pub use trigger::*;
pub use value::*;

use crate::cards::CardImpl;
use crate::effects::{k, KindMask};

pub struct CardSpec {
    /// Twinleaf behavior class (registry key, as `CardImpl::class`).
    pub class: &'static str,
    pub attacks: &'static [AttackSpec],
    pub powers: &'static [PowerSpec],
    pub play: Option<PlaySpec>,
    /// A Stadium's "once during each player's turn, that player may ..."
    /// (played with the board's use-Stadium action).
    pub use_stadium: Option<PlaySpec>,
    pub passives: &'static [Passive],
    pub triggers: &'static [Trigger],
}

impl CardSpec {
    pub const NONE: CardSpec = CardSpec { class: "", attacks: &[], powers: &[], play: None, use_stadium: None, passives: &[], triggers: &[] };

    /// The registry entry: the shared interpreter, subscribed to exactly the
    /// effect kinds this spec reacts to.
    pub const fn card_impl(&'static self) -> CardImpl {
        CardImpl { class: self.class, mask: self.mask(), reduce: run::reduce, resume: Some(run::resume), coin: Some(run::coin), can_play: None }
    }

    const fn mask(&self) -> KindMask {
        let mut m = KindMask::EMPTY;
        let mut i = 0;
        while i < self.attacks.len() {
            let steps = self.attacks[i].steps;
            let mut j = 0;
            while j < steps.len() {
                m = with(m, match steps[j].at {
                    RuleStep::AfterDamage => k::AFTER_ATTACK,
                    _ => k::ATTACK,
                });
                // The step-D choice pass of after-damage steps runs on the AttackEffect.
                m = with(m, k::ATTACK);
                j += 1;
            }
            i += 1;
        }
        if !self.powers.is_empty() {
            m = with(m, k::POWER);
        }
        let mut i = 0;
        while i < self.powers.len() {
            if matches!(self.powers[i].once, Once::PerTurn(_) | Once::PerTurnShared(_)) {
                // The once-per-turn marker is cleared at the end of the turn, and when
                // the card is played again (a new Pokémon).
                m = with(m, k::END_TURN);
                m = with(m, k::PLAY_POKEMON);
            }
            i += 1;
        }
        if self.use_stadium.is_some() {
            m = with(m, k::USE_STADIUM);
        }
        if self.play.is_some() {
            m = with(m, k::TRAINER);
        }
        let mut i = 0;
        while i < self.passives.len() {
            m = merge(m, passive::modifier_kinds(&self.passives[i].modifier));
            i += 1;
        }
        let mut i = 0;
        while i < self.triggers.len() {
            m = merge(m, trigger::event_kinds(&self.triggers[i].event));
            i += 1;
        }
        m
    }
}

pub(crate) const fn with(mut m: KindMask, kind: u32) -> KindMask {
    m.0[(kind >> 6) as usize] |= 1u64 << (kind & 63);
    m
}

const fn merge(mut a: KindMask, b: KindMask) -> KindMask {
    let mut i = 0;
    while i < 4 {
        a.0[i] |= b.0[i];
        i += 1;
    }
    a
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
    /// Once during your turn, per copy: the named player marker, set by this
    /// card when the Ability is used and cleared at the end of the turn.
    PerTurn(&'static str),
    /// Once during your turn for all copies together ("1 X per turn"): refused while any copy's
    /// marker named so is set; set by this card, cleared at the end of the turn.
    PerTurnShared(&'static str),
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
    /// An attack effect, carried out after the damage; its choices are made
    /// at step D, before the damage (`run.rs`).
    pub const fn after_damage(op: Op) -> Step {
        Step { at: RuleStep::AfterDamage, op }
    }
}

/// Where a step runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleStep {
    /// Attack text that changes the attack before the damage (damage
    /// modifiers, Joust-style text).
    BeforeDamage,
    /// Attack effects, carried out after the damage.
    AfterDamage,
    /// Trainer, Ability, trigger, or a step inside a nested list.
    Use,
}

/// Most items one step-D answer can record.
pub const SPEC_CHOICE_ITEMS: usize = 64;

/// An attack choice made at step D (before the damage) and carried out after
/// it: the step it belongs to and the answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecChoice {
    pub card: crate::list::CardId,
    /// Program and path of the step (`run.rs`).
    pub key: u64,
    pub answer: u8,
    /// The chosen items (card ids, slots as `p << 4 | slot`, counts), as the
    /// op encodes them; `len` of them are used. A hand can hold most of a
    /// deck, so this holds up to `SPEC_CHOICE_ITEMS`.
    pub items: [u8; SPEC_CHOICE_ITEMS],
    pub len: u8,
}

/// Every op of vocabulary v1. Record types live in the family files.
pub enum Op {
    // ops/cards.rs
    Move(MoveSpec),
    Pick(PickSpec),
    Draw(DrawSpec),
    Shuffle(ShuffleSpec),
    Reveal(RevealSpec),
    Search(SearchSpec),
    Snapshot(SnapshotSpec),
    Order(OrderSpec),
    Attach(AttachSpec),
    MoveEnergy(MoveEnergySpec),
    DiscardEnergy(DiscardEnergySpec),
    PlayFromZone(PlayFromZoneSpec),
    PickPrize(PickPrizeSpec),
    PrizeVisibility(PrizeVisibilitySpec),
    TakePrize(TakePrizeSpec),
    HandShuffleDraw(HandShuffleDrawSpec),
    // ops/board.rs
    PickSlot(PickSlotSpec),
    Switch(SwitchSpec),
    Heal(HealSpec),
    Damage(DamageSpec),
    DamageSlot(DamageSlotSpec),
    PlaceCounters(PlaceCountersSpec),
    SpreadCounters(SpreadCountersSpec),
    MoveCounters(MoveCountersSpec),
    Evolve(EvolveSpec),
    Devolve(DevolveSpec),
    SwapPokemonCard(SwapPokemonCardSpec),
    RemoveFromPlay(RemoveFromPlaySpec),
    Conditions(ConditionsSpec),
    KnockOut(KnockOutSpec),
    // ops/flow.rs
    Coin(CoinSpec),
    May(MaySpec),
    If(IfSpec),
    Choose(ChooseSpec),
    ForEach(ForEachSpec),
    Repeat(RepeatSpec),
    Parallel(ParallelSpec),
    Fail(FailSpec),
    PickAttack(PickAttackSpec),
    CopyAttack(CopyAttackSpec),
    EndTurn(EndTurnSpec),
    EndGame(EndGameSpec),
    Custom(CustomSpec),
    // ops/state.rs
    AttackFlag(AttackFlagSpec),
    SetMarker(SetMarkerSpec),
    ClearMarker(ClearMarkerSpec),
    Arm(ArmSpec),
    // S3 appends
    // ops/cards.rs
    PlayAsPokemon(PlayAsPokemonSpec),
    MoveEnergyOwn(MoveEnergyOwnSpec),
    // ops/board.rs
    SpreadDamage(SpreadDamageSpec),
    // ops/state.rs
    AbilityUsed(AbilityUsedSpec),
    SetFlag(SetFlagSpec),
    /// Damage to several of the opponent's Pokémon the attacker picks (min = max = the lesser of
    /// `count` and the Pokémon to pick from).
    /// This Pokémon (the slot `target`) switches with the Active Pokémon when it is on the Bench.
    SwitchWithActive(SwitchWithActiveSpec),
    /// Handheld Fan: the damaged Pokémon's owner moves an Energy from the Attacking Pokémon to another Benched
    /// Pokémon of the attacker's side (a trigger's context).
    // S3-4 appends (ops/board.rs)
    EachSlot(EachSlotSpec),
    ChoiceDamage(ChoiceDamageSpec),
    // S3-4 appends (ops/cards.rs)
    PrizeBonus(PrizeBonusSpec),
    BotherBot(BotherBotSpec),
    // S3-4 appends (ops/flow.rs)
}

/// Everything a spec card file needs.
pub mod prelude {
    pub use super::*;
    pub use crate::cards::CardImpl;
}
