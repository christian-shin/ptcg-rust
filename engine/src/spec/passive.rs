//! Passive modifiers (vocabulary v1 "Passives"): rules that apply while their
//! card is in place, by changing the effects flowing through the engine
//! (`reduce_effect`). A play or use blocker is a passive too, and doubles as
//! the blocker registry declared legality consults.
//!
//! Each modifier declares the effect kinds it reacts to (`modifier_kinds`,
//! used for the card's mask) and changes the effect in `apply`. Every copy of
//! the card is called for every effect of those kinds, in any zone: `locate`
//! decides whether this copy is in place, and `blocked` is the lock probe
//! derived from the passive's origin (Ability lock, Tool, Special Energy,
//! Stadium).

use super::*;
use crate::effects::{mask, AtkBase, EffId, Effect, EnergyEntry, KindMask, SlotRef};
use crate::game::{fx_flag, Game, R};
use crate::list::*;
use crate::prefabs::*;
use crate::state::{AttackRef, ListRef};
use crate::types::*;

/// A modifier that applies while its card is in place.
pub struct Passive {
    pub origin: RuleSource,
    pub modifier: Modifier,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RuleSource {
    Ability,
    /// A Pokémon Tool: applies to the Pokémon it is attached to, unless Tools
    /// have no effect.
    Tool,
    Energy,
    Stadium,
    TrainerEffect,
    CardRule,
}

pub enum Modifier {
    DamageDealt(DamageDealtSpec),
    DamageTaken(DamageTakenSpec),
    Prevent(PreventSpec),
    BlockUse(BlockUseSpec),
    AbilityLock(AbilityLockSpec),
    AttackCost(AttackCostSpec),
    RetreatCost(RetreatCostSpec),
    /// +HP to the Pokémon.
    HpBonus(i32),
    SurviveOnTen(SurviveOnTenSpec),
    ProvidesEnergy(ProvidesEnergySpec),
    PrizeAdjust(PrizeAdjustSpec),
    CheckupDamage(CheckupDamageSpec),
    GrantAttacks(GrantAttacksSpec),
    AttackFlags(AttackFlagsSpec),
    StatOverride(StatOverrideSpec),
    /// "Recovers from all Special Conditions" (Festival Grounds, Bubbly Water Energy): the Pokémon matching
    /// `subject` recover whenever the table state is checked (RemoveCondition events). "Can't be affected by" is
    /// a `Prevent` declaration over GainCondition.
    Recover(RecoverSpec),
    AttachGuard(AttachGuardSpec),
    BenchSize(BenchSizeSpec),
    // Appended by F-passive.
    /// +/- HP to the Pokémon matching a predicate (`HpBonus` with a subject and a guard).
    HpMod(HpModSpec),
    ProvidesEnergyBoost(ProvidesEnergyBoostSpec),
    // Appended by S3.
    NextTurnBonus(NextTurnBonusSpec),
    BenchAttacks(BenchAttacksSpec),
    /// `PrizeAdjust`, once per game for the Knocked Out Pokémon's player (Legacy Energy).
    PrizeAdjustOnce(PrizeAdjustSpec),
    /// The Pokémon matching `subject` has exactly these types (the game's type check).
    TypeOverride(TypeOverrideSpec),
    /// A permission (events design 4.2: "can evolve during ..."): while the card is in place for its origin
    /// and `while_` holds, the `lifts` limits don't apply to the events matching `for_`
    /// (`engine::enter::permitted`). Evaluated where the limit is checked; nothing is dispatched to it.
    Permit(PermitSpec),
    /// The Weakness of the opponent's Pokémon matching `subject` is `weakness` (Fairy Zone).
    WeaknessOverride(WeaknessOverrideSpec),
    /// An Ability lock that applies while this Pokémon is in the Active Spot, with the Ability lockers'
    /// activation order (a lock that was in effect first suppresses a later one).
    ActiveLock(ActiveLock),
    // --- S3-4 appends ---
    /// The card's Pokémon can't attack unless a condition holds.
    BlockAttack(BlockAttackSpec),
}

/// The Active-Spot Ability locks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveLock {
    /// Flutter Mane's Midnight Fluttering: your opponent's Active Pokémon has no Abilities, except for
    /// Midnight Fluttering ("Hide 'n' Sneak" takes precedence).
    MidnightFluttering,
    /// Iron Thorns ex's Initialization: while it is in either Active Spot, Pokémon with a Rule Box (yours and
    /// your opponent's) have no Abilities, except Future Pokémon.
    Initialization,
}

pub struct WeaknessOverrideSpec {
    /// The Pokémon whose Weakness changes (checked on the opponent's side only).
    pub subject: SlotPred,
    pub weakness: CardType,
}

pub struct TypeOverrideSpec {
    pub subject: SlotPred,
    pub set: &'static [CardType],
}

/// `Permit { origin, for, lifts, while_ }` (events design 4.2): the events matching `for_` (evaluated for the
/// declaring card: `This(Role::Base)` is "this Pokémon evolves") are free of the `lifts` limits on the rule
/// path. A card's own `Restrict` still applies (id1144, id1815).
pub struct PermitSpec {
    pub for_: super::event::EventPred,
    pub lifts: &'static [super::event::Limit],
    /// Conditions on the declaring card (`LockWhile::Active`: it is its owner's Active Pokémon).
    pub while_: &'static [LockWhile],
}
/// While this Pokémon is Active, it can use the attacks of any of the owner's Benched Pokémon
/// (Mew ex's Memory Helix): they are offered as copied attacks.
pub struct BenchAttacksSpec {}

// ---------------------------------------------------------------------------
// Records

/// Where in the damage calculation a bonus applies.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DamageStage {
    /// `DealDamage`: the damage to the Defending Pokémon, before Weakness and Resistance.
    Deal,
    /// `Attack`: the attack's damage, before it is dealt.
    Attack,
}

/// Whose attacks a passive reacts to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    /// Only the card's owner (the attacking player, or the damaged player).
    Owner,
    Any,
    /// Only the owner's opponent.
    Opponent,
}

/// The effects that do not stack: only the first reduction per effect applies.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NonStack {
    /// Bouffalant's Curly Wall: one per damage effect.
    CurlyWall,
}

impl NonStack {
    const fn flag(self) -> u8 {
        match self {
            NonStack::CurlyWall => fx_flag::CURLY_WALL,
        }
    }
}

/// "This attack does N more damage" (vocabulary P1).
pub struct DamageDealtSpec {
    pub stage: DamageStage,
    pub amount: i32,
    /// The attacking Pokémon.
    pub attacker: SlotPred,
    /// The attacking player is the card's owner (`Side::Owner`).
    pub side: Side,
    /// The damaged Pokémon: only the opponent's Active when true.
    pub opp_active_only: bool,
    /// The damaged Pokémon.
    pub target: SlotPred,
    /// The damage so far is above 0.
    pub needs_damage: bool,
    /// The attack's printed damage is above 0.
    pub needs_printed_damage: bool,
    pub guard: Cond,
    /// Doesn't stack: only the first bonus of this kind per damage effect applies (Hop's Snorlax).
    pub nonstacking: bool,
}

impl DamageDealtSpec {
    pub const DEFAULT: DamageDealtSpec = DamageDealtSpec {
        stage: DamageStage::Deal,
        amount: 0,
        attacker: SlotPred::Any,
        side: Side::Owner,
        opp_active_only: true,
        target: SlotPred::Any,
        needs_damage: false,
        needs_printed_damage: false,
        guard: Cond::True,
        nonstacking: false,
    };
}

/// "This Pokémon takes N less damage from attacks" (vocabulary P2).
pub struct DamageTakenSpec {
    pub amount: i32,
    /// The damaged Pokémon.
    pub subject: SlotPred,
    /// The attacking Pokémon.
    pub source: SlotPred,
    /// Only the card owner's Pokémon are damaged (`Side::Owner`).
    pub side: Side,
    /// Damage from any attack, the damaged player's own included (otherwise only the opponent's).
    pub from_any_attack: bool,
    pub guard: Cond,
    pub nonstacking: Option<NonStack>,
    /// A Tool that is discarded after it reduced damage.
    pub then_discard: bool,
}

impl DamageTakenSpec {
    pub const DEFAULT: DamageTakenSpec = DamageTakenSpec {
        amount: 0,
        subject: SlotPred::Holder,
        source: SlotPred::Any,
        side: Side::Any,
        from_any_attack: false,
        guard: Cond::True,
        nonstacking: None,
        then_discard: false,
    };
}

/// What a `Prevent` passive stops.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreventWhat {
    /// None of these: the declaration is over events (`PreventSpec::from`).
    None,
    /// Pokémon Tools have no effect (a Stadium): every Tool effect throws unless the Stadium's
    /// effect is blocked on the Pokémon holding it.
    ToolEffects,
}

/// A prevention (events design section 5: `Prevent { origin, protects: SlotPred, from: EventPred }`): the events
/// matching `from` don't happen to the Pokémon matching `protects` while the declaring card is in place for its
/// origin and its effect isn't blocked there ("this Pokémon can't be Confused", "your opponent's Active Pokémon
/// can't be healed", "can't be affected by any Special Conditions"). Both predicates are evaluated for the
/// declaring card, `protects` on the event's spot. The event's routine asks `derived::event_prevented` (events
/// batch 4: GainCondition, RemoveCondition, RemoveCounters, CoinFlip). `what` is the old form, a prohibition
/// not over events yet (`PreventWhat::None` for a declaration over events).
pub struct PreventSpec {
    pub what: PreventWhat,
    pub protects: SlotPred,
    pub from: super::event::EventPred,
    /// "Flip a coin; if heads, prevent that damage" (Fezandipiti's Adrena-Pheromone; user decision D8): the declaring
    /// card's owner flips, only for damage no other prevention stops and only when there is damage
    /// (`coin_prevented`, after `event_prevented`); the hard reader skips it.
    pub coin: bool,
}

impl PreventSpec {
    /// No prevention: write `..PreventSpec::NONE` for the fields a declaration doesn't use.
    pub const NONE: PreventSpec = PreventSpec { what: PreventWhat::None, protects: SlotPred::Any, from: super::event::EventPred::NEVER, coin: false };

    /// The events `from` don't happen to the Pokémon matching `protects`.
    pub const fn on(protects: SlotPred, from: super::event::EventPred) -> PreventSpec {
        PreventSpec { what: PreventWhat::None, protects, from, coin: false }
    }

    /// The events `from` don't happen to the Pokémon matching `protects` if the declaring card's owner flips heads.
    pub const fn on_coin(protects: SlotPred, from: super::event::EventPred) -> PreventSpec {
        PreventSpec { what: PreventWhat::None, protects, from, coin: true }
    }
}

impl std::fmt::Debug for PreventSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PreventSpec {{ what: {:?}, coin: {} }}", self.what, self.coin)
    }
}

/// The kinds a `Prevent` over events is listed under (the effect kinds of `from`, where the reader walks its
/// sources) and the marker its event's routine reads (`prevent_marker`): nothing for the old form.
const fn prevent_kinds(p: &PreventSpec) -> KindMask {
    if p.from.is_never() {
        return KindMask::EMPTY;
    }
    // The kinds the declaration ranges over (a cause-only one: every event with an effect done to a Pokémon, never
    // Damage), restricted to the families whose routine asks the reader, each with its marker.
    let all = p.from.prevent_kinds();
    let mut m = KindMask::EMPTY;
    let mut i = 0;
    while i < PREVENT_FAMILIES.len() {
        let (fam, marker) = PREVENT_FAMILIES[i];
        if all.intersects(fam) {
            m = m.or(and(all, fam));
            m = crate::spec::with(m, marker);
        }
        i += 1;
    }
    if prevention_recovers(p) {
        // The recovery while the protection is in force (`recover_protected`), at the table-state check.
        m = crate::spec::with(m, crate::effects::k::CHECK_TABLE_STATE);
    }
    m
}

const fn and(a: KindMask, b: KindMask) -> KindMask {
    KindMask([a.0[0] & b.0[0], a.0[1] & b.0[1], a.0[2] & b.0[2], a.0[3] & b.0[3]])
}

/// The event families whose routine asks the `Prevent` reader (`event_prevented`), each with the marker a prevention over
/// it sets (`prevent_kinds`) and the reader tests (`prevent_marker`).
const PREVENT_FAMILIES: [(KindMask, u32); 11] = [
    (super::event::ATTACH_EVENT_KINDS, crate::effects::k::DECLARES_ATTACH_PREVENT),
    (super::event::POKEMON_PREVENT_KINDS, crate::effects::k::DECLARES_POKEMON_PREVENT),
    (super::event::APPLY_EVENT_KINDS, crate::effects::k::DECLARES_APPLY_PREVENT),
    (super::event::KO_EVENT_KINDS, crate::effects::k::DECLARES_KO_PREVENT),
    (super::event::LEAVE_EVENT_KINDS, crate::effects::k::DECLARES_LEAVE_PREVENT),
    (super::event::DAMAGE_EVENT_KINDS, crate::effects::k::DECLARES_DAMAGE_PREVENT),
    (super::event::CONDITION_EVENT_KINDS, crate::effects::k::DECLARES_CONDITION_PREVENT),
    (super::event::HEAL_EVENT_KINDS, crate::effects::k::DECLARES_HEAL_PREVENT),
    (super::event::COIN_EVENT_KINDS, crate::effects::k::DECLARES_COIN_PREVENT),
    (super::event::ACTIVE_EVENT_KINDS, crate::effects::k::DECLARES_ACTIVE_PREVENT),
    (super::event::COUNTER_EVENT_KINDS, crate::effects::k::DECLARES_COUNTER_PREVENT),
];

/// Does some routine consult the `Prevent` reader for effects of this kind (the VERIFY cause cross-check: an opponent's
/// attack or Ability effect on a Pokémon no reader consults escapes every prevention)?
pub fn prevent_consulted(kind: u32) -> bool {
    PREVENT_FAMILIES.iter().any(|(fam, _)| fam.has(kind))
}

/// The event families a lock over events can forbid, each with the marker a lock over it sets (`block_kinds`) and the
/// query tests (`lock_marker`).
const LOCK_FAMILIES: [(KindMask, u32); 9] = [
    (super::event::PLAY_EVENT_KINDS, crate::effects::k::DECLARES_PLAY_LOCK),
    (super::event::CARD_EVENT_KINDS, crate::effects::k::DECLARES_CARD_LOCK),
    (super::event::POKEMON_EVENT_KINDS, crate::effects::k::DECLARES_EVENT_LOCK),
    (super::event::ATTACH_EVENT_KINDS, crate::effects::k::DECLARES_ATTACH_LOCK),
    (super::event::CONDITION_EVENT_KINDS, crate::effects::k::DECLARES_CONDITION_LOCK),
    (super::event::HEAL_EVENT_KINDS, crate::effects::k::DECLARES_HEAL_LOCK),
    (super::event::COIN_EVENT_KINDS, crate::effects::k::DECLARES_COIN_LOCK),
    (super::event::ACTIVE_EVENT_KINDS, crate::effects::k::DECLARES_ACTIVE_LOCK),
    (super::event::COUNTER_EVENT_KINDS, crate::effects::k::DECLARES_COUNTER_LOCK),
];
/// Whom a lock stops, relative to the owner of the lock's source.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Binds {
    /// The player who isn't the source's owner.
    Opponent,
    /// The source's owner.
    Owner,
    /// Either player.
    Both,
}

/// A condition on the lock's source that must hold for the lock to be on. The source also has to be in place
/// for its origin (a Pokémon with the Ability in play, the Stadium in play, ...), as for every passive.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LockWhile {
    /// The source is its owner's Active Pokémon.
    Active,
    /// The source's Pokémon has a Pokémon Tool attached.
    HasTool,
    /// The card the action uses is the source itself (the Pokémon evolving into it, the Active Pokémon
    /// that retreats, the Stadium being used).
    CardIsSource,
}

/// When a lock is a coin: "whenever they try to [do it], they flip a coin; if tails, [it doesn't happen and] they
/// discard that card instead" (Seismitoad's Quaking Fist; JP FAQ ガマゲロゲ). A coin-gated lock isn't a legality check
/// (heads lets the action through): `event_locked` doesn't answer it; the event's routine flips (`coin_gate`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoinGate {
    /// Not a coin: the lock forbids.
    No,
    /// Flip when trying: tails, the event doesn't happen and the card is discarded instead.
    TailsDiscardsCard,
}

/// A lock (events design section 5: `Lock { forbids }`), and the code the stopped action fails with: the events it
/// forbids (`forbids`), evaluated for the lock's source card; the locked player is the event's actor
/// (`EventView::actor`, the `Cause` player: id25, id230, id959). The in-play locks ([`BlockUseSpec`]) and the locks an
/// attack leaves on the opponent (`Lasting::OppCannotPlay`, stored on the locked player as a
/// [`crate::state::LastingLock`]) are this one declaration. Events batch 7: the play locks are over PlayTrainer ("your
/// opponent can't play Item cards from their hand": `Kind(PlayTrainer) & Use(Played) & Source(Hand) & Card(Item)`).
pub struct LockDecl {
    pub error: &'static str,
    pub forbids: super::event::EventPred,
    pub coin: CoinGate,
}

impl LockDecl {
    /// No events (with `..LockDecl::NONE`).
    pub const NONE: LockDecl = LockDecl { error: "BLOCKED_BY_EFFECT", forbids: super::event::EventPred::NEVER, coin: CoinGate::No };

    /// A lock on the events matching `forbids`, failing with `error`.
    pub const fn on(forbids: super::event::EventPred, error: &'static str) -> LockDecl {
        LockDecl { error, forbids, coin: CoinGate::No }
    }

    /// Two locks that are one declaration (so one stands for both).
    pub fn same_as(&self, o: &LockDecl) -> bool {
        std::ptr::eq(self, o)
    }
}

/// "Your opponent can't play Item cards from their hand" (Tyranitar, Jellicent ex's Items half, Budew, Frillish,
/// Galvantula ex): PlayTrainer of an Item played from the hand.
pub const PLAY_ITEM_FROM_HAND: super::event::EventPred = super::event::EventPred::All(&[
    super::event::EventPred::Kind(super::event::EventKind::PlayTrainer),
    super::event::EventPred::Use(super::event::TrainerUse::Played),
    super::event::EventPred::Source(super::event::RulesZone::Hand),
    super::event::EventPred::Card(Pred::Item),
]);
/// "... can't play Supporter cards from their hand" (Scream Tail ex).
pub const PLAY_SUPPORTER_FROM_HAND: super::event::EventPred = super::event::EventPred::All(&[
    super::event::EventPred::Kind(super::event::EventKind::PlayTrainer),
    super::event::EventPred::Use(super::event::TrainerUse::Played),
    super::event::EventPred::Source(super::event::RulesZone::Hand),
    super::event::EventPred::Card(Pred::Supporter),
]);
/// "... can't play Stadium cards from their hand" (Chi-Yu MEG).
pub const PLAY_STADIUM_FROM_HAND: super::event::EventPred = super::event::EventPred::All(&[
    super::event::EventPred::Kind(super::event::EventKind::PlayTrainer),
    super::event::EventPred::Use(super::event::TrainerUse::Played),
    super::event::EventPred::Source(super::event::RulesZone::Hand),
    super::event::EventPred::Card(Pred::Stadium),
]);
/// "... use a Trainer card from their hand" (Seismitoad's Quaking Fist): PlayTrainer of any Trainer played from the hand.
pub const PLAY_TRAINER_FROM_HAND: super::event::EventPred = super::event::EventPred::All(&[
    super::event::EventPred::Kind(super::event::EventKind::PlayTrainer),
    super::event::EventPred::Use(super::event::TrainerUse::Played),
    super::event::EventPred::Source(super::event::RulesZone::Hand),
]);

impl PartialEq for LockDecl {
    fn eq(&self, o: &LockDecl) -> bool {
        std::ptr::eq(self, o)
    }
}
impl Eq for LockDecl {}
impl std::fmt::Debug for LockDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LockDecl({}, {:?})", self.error, self.coin)
    }
}

/// A lock of a card in play, declared as data (`event_locked` evaluates it for execution and for
/// legality): the players it `binds`, what it stops (`lock`), the source's conditions (`while_`), and whether
/// the source's Ability has to be on (`ability`: a lock whose source has no Ability is off, so Jellicent ex's
/// Item lock ends when Iron Thorns ex removes its Ability).
pub struct BlockUseSpec {
    pub binds: Binds,
    pub lock: LockDecl,
    pub while_: &'static [LockWhile],
    pub ability: bool,
}

impl BlockUseSpec {
    /// "This card can't be put into your hand or deck from the discard pile" (Poké Vital A, Neutralization Zone): a lock
    /// over this card's PutIntoHand and PutIntoDeck from the discard pile (events batch 7), binding both players (the
    /// card's own rule, from the discard pile: origin `CardRule`).
    pub const NOT_FROM_DISCARD_TO_HAND_OR_DECK: BlockUseSpec = BlockUseSpec {
        binds: Binds::Both,
        lock: LockDecl::on(
            super::event::EventPred::All(&[
                super::event::EventPred::Any(&[super::event::EventPred::Kind(super::event::EventKind::PutIntoHand), super::event::EventPred::Kind(super::event::EventKind::PutIntoDeck)]),
                super::event::EventPred::This(super::event::Role::Card),
                super::event::EventPred::Source(super::event::RulesZone::Discard),
            ]),
            "BLOCKED_BY_EFFECT",
        ),
        while_: &[],
        ability: false,
    };
    /// "This Pokémon can't retreat" (the Antique Fossils): a lock over the ChangeActive of a retreat whose leaving
    /// Pokémon is this one (events batch 5). Retreating only: it can still be switched (APR C-03).
    pub const RETREAT_THIS_ACTIVE: BlockUseSpec = BlockUseSpec {
        binds: Binds::Owner,
        lock: LockDecl::on(
            super::event::EventPred::All(&[
                super::event::EventPred::Kind(super::event::EventKind::ChangeActive),
                super::event::EventPred::Change(super::event::ActiveChange::Retreat),
                super::event::EventPred::From(SlotPred::IsThisPokemon),
            ]),
            "CANNOT_RETREAT",
        ),
        while_: &[],
        ability: false,
    };
}

/// The kinds a lock's source is listed under (its dispatch mask: `event_locked` walks the sources of the event's
/// kind), plus the marker of a lock over events.
const fn block_kinds(lock: &LockDecl) -> KindMask {
    let mut m = KindMask::EMPTY;
    if !lock.forbids.is_never() {
        // The kinds it forbids, restricted to the families the lock query is asked for, each with its marker.
        let all = lock.forbids.effect_kinds();
        let mut i = 0;
        while i < LOCK_FAMILIES.len() {
            let (fam, marker) = LOCK_FAMILIES[i];
            if all.intersects(fam) {
                m = m.or(and(all, fam));
                m = crate::spec::with(m, marker);
            }
            i += 1;
        }
    }
    m
}

/// Where the locking card must be.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Locker {
    /// The top Pokémon of a Benched slot of either player.
    BenchOfEitherSide,
    /// The top Pokémon of any slot in play, either player's.
    InPlayEitherSide,
    /// The Stadium in play.
    StadiumInPlay,
}

/// The lock probe that decides whether the locking card's own effect applies.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LockerProbe {
    /// The locker's Ability is not itself blocked.
    Generic,
    /// A real use of the locker's own power `n` is not blocked.
    OwnPower(u8),
    /// The Stadium's effect is not blocked on the checked Pokémon's slot.
    StadiumOnSlot,
}

/// Which powers a lock strips.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LockedPowers {
    /// The plain Ability lock probe is subject too.
    pub generic_probe: bool,
    /// Only Abilities that require the Pokémon to Knock Out itself.
    pub only_knocks_out_self: bool,
    pub exempt_name: Option<&'static str>,
}

/// "Pokémon ... have no Abilities" (vocabulary P7): strips matching Abilities and blocks their use.
pub struct AbilityLockSpec {
    /// The error of a blocked use.
    pub error: &'static str,
    pub locker: Locker,
    /// The card whose Abilities are checked.
    pub card: Pred,
    /// The Pokémon slot that card is in (a card in no Pokémon slot is never locked, unless `missing`).
    pub slot: SlotPred,
    /// A card in no list at all: locked when this holds (its printed data).
    pub missing: Pred,
    pub powers: LockedPowers,
    pub probe: LockerProbe,
}
/// A change of an attack's or a retreat's cost.
pub enum CostChange {
    /// Costs this much [C] less (the reductions add up and apply once, with the increases).
    Reduce(Num),
    /// Costs [C] more.
    Add(i32),
    /// No cost at all (retreat).
    Free,
    /// Ignore all [C] in the cost, those added later included (attack).
    IgnoreColorless,
    /// "Can use this attack for ...": the cost is set to exactly these types (attack).
    SetCost(&'static [CardType]),
    /// Costs 1 Energy of any type less (applied by the core with the other cost changes).
    AnyOne,
}

/// A change of the cost of the attacks of a Pokémon (vocabulary P8).
pub struct AttackCostSpec {
    pub change: CostChange,
    /// Only this attack of the card (otherwise any attack of the Pokémon).
    pub attack: Option<u8>,
    /// The attacking player's Active Pokémon.
    pub subject: SlotPred,
    /// The attacking player is the card's owner.
    pub side: Side,
    pub guard: Cond,
}

impl AttackCostSpec {
    pub const DEFAULT: AttackCostSpec = AttackCostSpec { change: CostChange::Free, attack: None, subject: SlotPred::Holder, side: Side::Any, guard: Cond::True };
}

/// Which Active Pokémon a retreat cost change looks at.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RetreatWhich {
    /// The retreating player's Active Pokémon.
    Mine,
    /// Either player's Active Pokémon (the change applies when one matches).
    Either,
}

/// A change of a retreat cost (vocabulary P9).
pub struct RetreatCostSpec {
    pub change: CostChange,
    pub which: RetreatWhich,
    pub subject: SlotPred,
    /// The retreating player is the card's owner.
    pub side: Side,
    pub guard: Cond,
}

impl RetreatCostSpec {
    pub const DEFAULT: RetreatCostSpec = RetreatCostSpec { change: CostChange::Free, which: RetreatWhich::Mine, subject: SlotPred::Holder, side: Side::Any, guard: Cond::True };
}
/// "If this Pokémon would be Knocked Out by damage from an attack, it survives with 10 HP left"
/// (Tenacious Body / Tenacious Heart), under a condition.
pub struct SurviveOnTenSpec {
    pub kind: SurviveKind,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SurviveKind {
    /// A coin flip decides (flipped once the attack's damage is all done).
    OnCoin,
    /// Only while the Pokémon has no damage counters (full HP).
    IfFullHp,
    /// A Pokémon Tool: while the Pokémon has no damage counters and would be Knocked Out by damage from an
    /// attack of the opponent's Pokémon, it survives with 10 HP left and the Tool is discarded.
    ToolIfFullHp,
}
/// What the Energy provides, as one entry of the Energy map.
pub struct ProvidesEnergySpec {
    /// One Energy map entry per element whose condition holds for the Pokémon.
    pub entries: &'static [ProvidedEntry],
    /// Skip when a Special Energy lock probe fails (otherwise pushed unconditionally).
    pub probe: bool,
}

/// An entry of the Energy map: the types of one unit of Energy provided.
pub struct ProvidedEntry {
    pub when: SlotPred,
    pub provides: &'static [CardType],
}

impl ProvidedEntry {
    pub const fn always(provides: &'static [CardType]) -> ProvidedEntry {
        ProvidedEntry { when: SlotPred::Any, provides }
    }
}

/// Each attached Energy card matching `energy` that provides nothing yet provides `provides`
/// (Meganium's Wild Growth).
pub struct ProvidesEnergyBoostSpec {
    pub energy: Pred,
    pub provides: &'static [CardType],
}
/// "Takes N fewer/more Prize cards" for a Knock Out (vocabulary P13).
pub struct PrizeAdjustSpec {
    pub delta: i32,
    /// The Knocked Out Pokémon.
    pub subject: SlotPred,
    /// Only a Knock Out by damage from an attack.
    pub by_attack_damage: bool,
    /// The Knock Out is the owner's opponent's, by the attack named so of this very card
    /// (Unown's Mysterious Signal): the card's own last attack of the turn.
    pub by_own_attack: Option<&'static str>,
    pub guard: Cond,
    // --- S3 agent 3 appends ---
    /// The Pokémon that used the attack (by Knock Out by attack damage), where it is now.
    pub attacker: SlotPred,
    /// Applies once per Knocked Out Pokémon: this marker (a slot marker, by name) is set on it.
    pub nonstacking: Option<&'static str>,
    /// Only a Knock Out of the card owner's Pokémon.
    pub owner_only: bool,
}
impl PrizeAdjustSpec {
    pub const DEFAULT: PrizeAdjustSpec = PrizeAdjustSpec {
        delta: 0,
        subject: SlotPred::Any,
        by_attack_damage: false,
        by_own_attack: None,
        guard: Cond::True,
        attacker: SlotPred::Any,
        nonstacking: None,
        owner_only: false,
    };
}
/// "During Pokémon Checkup, put N more damage counters on each Poisoned Pokémon ..." (Perilous
/// Jungle, Pecharunt): added to the Poison damage of the Active Pokémon of the player whose
/// Checkup it is.
pub struct CheckupDamageSpec {
    /// More damage, in HP (10 per counter).
    pub amount: i32,
    /// The checked Pokémon.
    pub victim: SlotPred,
    /// Only the Checkup of the card owner's opponent.
    pub opponent_only: bool,
    /// The card's own Pokémon.
    pub holder: SlotPred,
    /// The damage is Burn damage (otherwise Poison damage).
    pub burn: bool,
}
/// The attacks of the earlier Evolutions in the slot are also this evolved Active Pokémon's
/// (Relicanth's Memory Dive).
pub struct GrantAttacksSpec {}
/// Flags the card writes on attacks: "if you go first, this Pokémon can attack on your first turn"
/// (`first_turn`), and Shred: the attacks of Pokémon matching `shred` ignore the effects on the Defending
/// Pokémon, while the card's Ability works.
pub struct AttackFlagsSpec {
    pub first_turn: bool,
    pub shred: Option<SlotPred>,
}
/// "During your next turn, this Pokémon's `attack` attack does `bonus` more damage" (S3 agent 3):
/// using the attack arms the bonus (it rolls over at the end of the turn), and every attack of
/// the Pokémon applies an armed one.
pub struct NextTurnBonusSpec {
    pub attack: &'static str,
    pub bonus: i32,
}
pub struct StatOverrideSpec {}
/// "Recovers from all Special Conditions": the Pokémon matching `subject` recover from `conds` (empty: all of
/// them) whenever the table state is checked.
pub struct RecoverSpec {
    pub conds: &'static [SpecialCondition],
    pub subject: SlotPred,
}
/// The Energy can only be attached to a matching Pokémon (and is discarded from any other).
pub struct AttachGuardSpec {
    pub allow: SlotPred,
}
/// Each player whose `guard` holds can have this many Benched Pokémon (the Stadium's rule); the
/// others keep the usual 5.
pub struct BenchSizeSpec {
    pub size: u8,
    pub guard: Cond,
}

/// +/- HP to the Pokémon matching `subject` while the card is in place.
pub struct HpModSpec {
    pub amount: i32,
    pub subject: SlotPred,
    pub guard: Cond,
}

/// The effect kinds a modifier reacts to.
pub const fn modifier_kinds(m: &Modifier) -> KindMask {
    use crate::effects::k;
    match m {
        Modifier::HpBonus(_) | Modifier::HpMod(_) => mask(&[k::CHECK_HP]),
        Modifier::DamageDealt(d) => match d.stage {
            DamageStage::Deal => mask(&[k::DEAL_DAMAGE]),
            DamageStage::Attack => mask(&[k::ATTACK]),
        },
        Modifier::DamageTaken(_) => mask(&[k::PUT_DAMAGE]),
        Modifier::SurviveOnTen(_) => mask(&[k::PUT_DAMAGE]),
        Modifier::CheckupDamage(_) => mask(&[k::BETWEEN_TURNS]),
        Modifier::NextTurnBonus(_) => mask(&[k::ATTACK]),
        Modifier::BenchAttacks(_) => mask(&[k::CHECK_POKEMON_ATTACKS]),
        Modifier::AttackFlags(a) => {
            if a.first_turn {
                mask(&[k::USE_ATTACK])
            } else {
                mask(&[k::ATTACK])
            }
        }
        Modifier::BlockUse(b) => block_kinds(&b.lock),
        Modifier::ProvidesEnergy(_) | Modifier::ProvidesEnergyBoost(_) => mask(&[k::CHECK_PROVIDED_ENERGY]),
        // The attach refusal is a check of the Attach routine (`engine::attach::check_attach_with`).
        Modifier::AttachGuard(_) => mask(&[k::CHECK_TABLE_STATE]),
        Modifier::Recover(_) => mask(&[k::CHECK_TABLE_STATE]),
        Modifier::AbilityLock(l) => {
            if lock_reads_attached(l) {
                mask(&[k::CHECK_POKEMON_POWERS, k::POWER, k::DECLARES_ATTACHED_LOCK])
            } else {
                mask(&[k::CHECK_POKEMON_POWERS, k::POWER])
            }
        }
        Modifier::Prevent(p) => prevent_kinds(p).or(match p.what {
            PreventWhat::None => KindMask::EMPTY,
            PreventWhat::ToolEffects => mask(&[k::TOOL]),
        }),
        Modifier::PrizeAdjust(_) | Modifier::PrizeAdjustOnce(_) => mask(&[k::KNOCK_OUT]),
        Modifier::TypeOverride(_) => mask(&[k::CHECK_POKEMON_TYPE]),
        Modifier::WeaknessOverride(_) => mask(&[k::CHECK_POKEMON_STATS]),
        Modifier::ActiveLock(ActiveLock::MidnightFluttering) => mask(&[k::CHECK_POKEMON_POWERS, k::POWER]),
        Modifier::ActiveLock(ActiveLock::Initialization) => mask(&[k::CHECK_POKEMON_POWERS, k::POWER, k::EFFECT_OF_ABILITY]),
        Modifier::Permit(pm) => {
            let mut m = mask(&[k::DECLARES_PERMIT]);
            let mut i = 0;
            while i < pm.lifts.len() {
                m = crate::spec::with(m, crate::engine::enter::limit_kind(pm.lifts[i]));
                i += 1;
            }
            m
        }
        Modifier::GrantAttacks(_) => mask(&[k::CHECK_POKEMON_ATTACKS]),
        Modifier::AttackCost(_) => mask(&[k::CHECK_ATTACK_COST]),
        Modifier::BlockAttack(b) => match b.on {
            AttackBlockOn::ActiveAttack => mask(&[k::ATTACK]),
            AttackBlockOn::UseAttack => mask(&[k::USE_ATTACK]),
        },
        Modifier::RetreatCost(_) => mask(&[k::CHECK_RETREAT_COST]),
        Modifier::BenchSize(_) => mask(&[k::CHECK_TABLE_STATE]),
        _ => KindMask::EMPTY,
    }
}

// ---------------------------------------------------------------------------
// Where the card is, and the lock probe

/// Where this copy of the card is, when it is in place for its origin.
#[derive(Clone, Copy)]
pub(crate) struct Located {
    /// The player whose lock probe applies.
    pub(crate) owner: usize,
    /// The Pokémon the card is part of or attached to.
    pub(crate) held: Option<SlotRef>,
}

fn slot_where(g: &Game, f: impl Fn(&crate::state::Slot, usize, u8) -> bool) -> Option<SlotRef> {
    // The Pokémon in play, Active first (`Player::in_play`), without building the list.
    for p in 0..2 {
        let pl = &g.st.players[p];
        for &s in std::iter::once(&pl.active).chain(pl.bench.iter()) {
            let slot = &pl.slots[s as usize];
            if !slot.cards.is_empty() && f(slot, p, s) {
                return Some(SlotRef::new(p, s));
            }
        }
    }
    None
}

pub(crate) fn locate(g: &Game, me: CardId, origin: RuleSource) -> Option<Located> {
    match origin {
        RuleSource::Tool => slot_where(g, |sl, _, _| sl.tools.contains(me)).map(|s| Located { owner: s.p as usize, held: Some(s) }),
        RuleSource::Energy => slot_where(g, |sl, _, _| sl.cards.contains(me) && !sl.tools.contains(me)).map(|s| Located { owner: s.p as usize, held: Some(s) }),
        RuleSource::Ability => slot_where(g, |sl, p, s| sl.cards.contains(me) && g.st.slot_pokemon(p, s) == Some(me)).map(|s| Located { owner: s.p as usize, held: Some(s) }),
        RuleSource::Stadium => {
            if g.st.stadium_card() == Some(me) {
                let owner = g.st.locate(me).and_then(|l| l.owner()).unwrap_or_else(|| g.st.owner(me));
                Some(Located { owner, held: None })
            } else {
                None
            }
        }
        RuleSource::TrainerEffect | RuleSource::CardRule => {
            let owner = g.st.locate(me).and_then(|l| l.owner()).unwrap_or_else(|| g.st.owner(me));
            Some(Located { owner, held: None })
        }
    }
}

/// Is the card's effect off: the lock probe derived from its origin. `affected`
/// is the Pokémon the effect is about (a Stadium's probe is per Pokémon).
pub(crate) fn blocked(g: &mut Game, me: CardId, origin: RuleSource, at: Located, affected: Option<SlotRef>) -> bool {
    match origin {
        RuleSource::Ability => is_ability_blocked(g, at.owner, me, None),
        RuleSource::Tool => is_tool_blocked(g, at.owner, me),
        RuleSource::Energy => match at.held {
            Some(held) => is_special_energy_blocked(g, at.owner, me, held, false),
            None => false,
        },
        RuleSource::Stadium => match affected {
            Some(t) => is_stadium_effect_blocked(g, t.p as usize, t, me),
            None => false,
        },
        RuleSource::TrainerEffect | RuleSource::CardRule => false,
    }
}

fn guard_ok(g: &Game, me: CardId, origin: RuleSource, owner: usize, guard: &Cond) -> bool {
    let f = run::Frame::passive(g, me, owner, origin);
    cond(g, me, &f, guard)
}

fn is_attack_phase(g: &Game) -> bool {
    g.st.phase == GamePhase::Attack
}

// ---------------------------------------------------------------------------
// Apply

pub(crate) fn apply(g: &mut Game, me: CardId, e: EffId, ps: &Passive) -> R {
    match &ps.modifier {
        Modifier::HpBonus(n) => hp_mod(g, me, e, ps.origin, *n, &SlotPred::Holder, &Cond::True),
        Modifier::HpMod(h) => hp_mod(g, me, e, ps.origin, h.amount, &h.subject, &h.guard),
        Modifier::SurviveOnTen(s) => survive_on_ten(g, me, e, ps.origin, s),
        Modifier::CheckupDamage(c) => checkup_damage(g, me, e, ps.origin, c),
        Modifier::NextTurnBonus(b) => next_turn_bonus(g, me, e, b),
        Modifier::BenchAttacks(_) => bench_attacks(g, me, e, ps.origin),
        Modifier::AttackFlags(a) => attack_flags(g, me, e, ps.origin, a),
        Modifier::DamageDealt(d) => damage_dealt(g, me, e, ps.origin, d),
        Modifier::DamageTaken(d) => damage_taken(g, me, e, ps.origin, d),
        // Locks are read by the event's routine and legality (`event_locked`); nothing is dispatched to them.
        Modifier::BlockUse(_) => Ok(()),
        Modifier::ProvidesEnergy(pe) => provides_energy(g, me, e, pe),
        Modifier::ProvidesEnergyBoost(b) => provides_energy_boost(g, me, e, ps.origin, b),
        Modifier::AttachGuard(a) => attach_guard(g, me, e, a),
        Modifier::AttackCost(c) => attack_cost(g, me, e, ps.origin, c),
        Modifier::Recover(c) => recover(g, me, e, ps.origin, c),
        Modifier::AbilityLock(l) => ability_lock(g, me, e, l),
        Modifier::Prevent(p) => prevent(g, me, e, ps.origin, p),
        Modifier::TypeOverride(t) => {
            let Effect::CheckPokemonType { target, .. } = *g.e(e) else { return Ok(()) };
            let Some(at) = locate(g, me, ps.origin) else { return Ok(()) };
            if !slot_pred_m(g, me, target, &t.subject)? || blocked(g, me, ps.origin, at, Some(target)) {
                return Ok(());
            }
            if let Effect::CheckPokemonType { card_types, .. } = g.e_mut(e) {
                card_types.clear();
                for ty in t.set.iter() {
                    card_types.push(*ty);
                }
            }
            Ok(())
        }
        // Permissions are read where a limit is checked (`engine::enter::permitted`); nothing is dispatched to them.
        Modifier::Permit(_) => Ok(()),
        Modifier::ActiveLock(l) => active_lock(g, me, e, *l),
        Modifier::WeaknessOverride(w) => {
            let Effect::CheckPokemonStats { target, .. } = *g.e(e) else { return Ok(()) };
            let player = target.p as usize;
            // Today's behavior: the Ability's lock probe is made for the checked Pokémon's owner, first.
            if is_ability_blocked(g, player, me, None) {
                return Ok(());
            }
            let Some(at) = locate(g, me, ps.origin) else { return Ok(()) };
            if at.owner == player || !slot_pred_m(g, me, target, &w.subject)? {
                return Ok(());
            }
            let (fx, _) = g.run_fx(Effect::EffectOfAbility { p: at.owner as u8, power: crate::effects::PowerRef { card: me, index: 0 }, card: me, target: Some(target), cause: crate::cause::Cause::of_origin(ps.origin, me, at.owner as u8) })?;
            if let Effect::EffectOfAbility { target: Some(_), .. } = fx {
                if let Effect::CheckPokemonStats { weakness, .. } = g.e_mut(e) {
                    weakness.clear();
                    weakness.push(crate::effects::WeaknessV { card_type: w.weakness, value: None });
                }
            }
            Ok(())
        }
        Modifier::PrizeAdjust(d) => prize_adjust(g, me, e, ps.origin, d),
        Modifier::PrizeAdjustOnce(d) => {
            let Effect::KnockOut { p, .. } = *g.e(e) else { return Ok(()) };
            let p = p as usize;
            if g.st.players[p].legacy_energy_used {
                return Ok(());
            }
            let before = match *g.e(e) {
                Effect::KnockOut { prize_count, .. } => prize_count,
                _ => 0,
            };
            prize_adjust(g, me, e, ps.origin, d)?;
            if matches!(*g.e(e), Effect::KnockOut { prize_count, .. } if prize_count != before) {
                g.st.players[p].legacy_energy_used = true;
            }
            Ok(())
        }
        Modifier::GrantAttacks(_) => grant_attacks(g, me, e, ps.origin),
        Modifier::RetreatCost(c) => retreat_cost(g, me, e, ps.origin, c),
        Modifier::BenchSize(d) => bench_size(g, me, e, ps.origin, d),
        Modifier::BlockAttack(b) => block_attack(g, me, e, ps.origin, b),
        _ => unimplemented!("spec passive not implemented yet (passive.rs)"),
    }
}

fn hp_mod(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, n: i32, subject: &SlotPred, guard: &Cond) -> R {
    let (target, card) = match *g.e(e) {
        Effect::CheckHp { target, card, .. } => (target, card),
        _ => return Ok(()),
    };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if !slot_pred_m(g, me, target, subject)? || blocked(g, me, origin, at, Some(target)) || !guard_ok_m(g, me, origin, at.owner, guard)? {
        return Ok(());
    }
    // HP is only changed for a Pokémon actually being checked (`effect.hp += n` writes only then).
    if card.is_some() {
        g.st.players[target.p as usize].slots[target.s as usize].hp_bonus += n;
    }
    Ok(())
}

fn damage_dealt(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &DamageDealtSpec) -> R {
    // (player, source slot, damaged slot, damage so far, the attack)
    let (player, source, target, damage, attack) = match (d.stage, *g.e(e)) {
        (DamageStage::Deal, Effect::DealDamage { b, damage }) => (b.player as usize, b.source, Some(b.target), damage, b.attack),
        (DamageStage::Attack, Effect::Attack { p, source, damage, attack, .. }) => (p as usize, source, None, damage, attack),
        _ => return Ok(()),
    };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if d.side == Side::Owner && at.owner != player {
        return Ok(());
    }
    let opp = 1 - player;
    if let Some(t) = target {
        if d.opp_active_only && !(t.p as usize == opp && t.s == g.st.players[opp].active) {
            return Ok(());
        }
    }
    if d.needs_damage && damage <= 0 {
        return Ok(());
    }
    if d.needs_printed_damage && crate::engine::attack::attack_def(g, attack).damage <= 0 {
        return Ok(());
    }
    let affected = target.unwrap_or(SlotRef::new(opp, g.st.players[opp].active));
    if blocked(g, me, origin, at, Some(affected)) {
        return Ok(());
    }
    if !slot_pred_m(g, me, source, &d.attacker)? || !guard_ok_m(g, me, origin, at.owner, &d.guard)? {
        return Ok(());
    }
    if let Some(t) = target {
        if !slot_pred_m(g, me, t, &d.target)? {
            return Ok(());
        }
    }
    if d.nonstacking && g.fx_flags(e) & fx_flag::DAMAGE_INCREASED != 0 {
        return Ok(());
    }
    match g.e_mut(e) {
        Effect::DealDamage { damage, .. } | Effect::Attack { damage, .. } => *damage += d.amount,
        _ => {}
    }
    if d.nonstacking {
        g.set_fx_flag(e, fx_flag::DAMAGE_INCREASED);
    }
    Ok(())
}

/// The preconditions both "takes less" and "prevent" share: damage put on a
/// Pokémon by an attack of the opponent's Pokémon, during the attack phase,
/// not ignored by Shred. Returns the attack base and the card's place.
fn damage_taken_prelude(
    g: &mut Game,
    me: CardId,
    e: EffId,
    origin: RuleSource,
    side: Side,
    subject: &SlotPred,
    source: &SlotPred,
    guard: &Cond,
    from_any_attack: bool,
) -> R<Option<(AtkBase, Located)>> {
    let b = match *g.e(e) {
        Effect::PutDamage { b, .. } | Effect::DealDamage { b, .. } => b,
        _ => return Ok(None),
    };
    if ignores_defender_effects(g, &b) || !is_attack_phase(g) {
        return Ok(None);
    }
    let t = b.target;
    if b.source.p == t.p && !from_any_attack {
        return Ok(None);
    }
    let Some(at) = locate(g, me, origin) else { return Ok(None) };
    if side == Side::Owner && at.owner != t.p as usize {
        return Ok(None);
    }
    if !slot_pred_m(g, me, t, subject)? || blocked(g, me, origin, at, Some(t)) {
        return Ok(None);
    }
    if !guard_ok(g, me, origin, at.owner, guard) || !slot_pred_m(g, me, b.source, source)? {
        return Ok(None);
    }
    Ok(Some((b, at)))
}

fn damage_taken(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &DamageTakenSpec) -> R {
    if !matches!(*g.e(e), Effect::PutDamage { .. }) {
        return Ok(());
    }
    let Some((b, _)) = damage_taken_prelude(g, me, e, origin, d.side, &d.subject, &d.source, &d.guard, d.from_any_attack)? else { return Ok(()) };
    if let Some(ns) = d.nonstacking {
        if g.fx_flags(e) & ns.flag() != 0 {
            return Ok(());
        }
    }
    if d.then_discard {
        // "If the Pokémon is damaged": with no damage taken the Tool stays.
        let damage_now = match *g.e(e) {
            Effect::PutDamage { damage, .. } => damage,
            _ => 0,
        };
        if damage_now <= 0 || crate::engine::damage::would_be_prevented(g, &b, damage_now)? {
            return Ok(());
        }
    }
    if let Effect::PutDamage { damage, .. } = g.e_mut(e) {
        *damage -= d.amount;
    }
    if let Some(ns) = d.nonstacking {
        g.set_fx_flag(e, ns.flag());
    }
    if d.then_discard {
        // "Then, discard this card": the Tool leaves play by its own text (a LeavePlay of the attached card, user decision
        // D1).
        let t = b.target;
        let cause = crate::cause::Cause::of_origin(origin, me, t.p);
        crate::engine::knockout::leave_play_cards(g, t, &[me], super::event::RulesZone::Discard, cause, None)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Energy

fn provides_energy(g: &mut Game, me: CardId, e: EffId, pe: &ProvidesEnergySpec) -> R {
    let (p, source) = match *g.e(e) {
        Effect::CheckProvidedEnergy { p, source, .. } => (p, source),
        _ => return Ok(()),
    };
    if !g.st.slot(source.p as usize, source.s).cards.contains(me) {
        return Ok(());
    }
    if pe.probe && g.run_fx(Effect::Energy { p, card: me }).is_err() {
        return Ok(());
    }
    for entry in pe.entries {
        if !slot_pred_m(g, me, source, &entry.when)? {
            continue;
        }
        let mut provides = SVec::new();
        for t in entry.provides {
            provides.push(*t);
        }
        if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
            energy_map.push(EnergyEntry { card: me, provides });
        }
    }
    Ok(())
}

fn provides_energy_boost(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, b: &ProvidesEnergyBoostSpec) -> R {
    let (p, source) = match *g.e(e) {
        Effect::CheckProvidedEnergy { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if at.owner != p || blocked(g, me, origin, at, Some(source)) {
        return Ok(());
    }
    let cards: Vec<CardId> = g.st.slot(source.p as usize, source.s).cards.iter().collect();
    for c in cards {
        if !pred(g, c, &b.energy) {
            continue;
        }
        if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(e) {
            if energy_map.iter().any(|m| m.card == c) {
                continue;
            }
            let mut provides = SVec::new();
            for t in b.provides {
                provides.push(*t);
            }
            energy_map.push(EnergyEntry { card: c, provides });
        }
    }
    Ok(())
}

/// May the Energy `me` be attached to `target` (its own `AttachGuard`)?
pub(crate) fn attach_guard_allows(g: &mut Game, me: CardId, target: SlotRef, a: &AttachGuardSpec) -> R<bool> {
    slot_pred_m(g, me, target, &a.allow)
}

/// Is attaching the Energy card `card` to `target` stopped by its own `AttachGuard` (the Attach routine refuses
/// it with `CANNOT_PLAY_THIS_CARD`, `engine::attach::check_attach_with`; legality asks the same)?
pub fn attach_guard_refuses(g: &mut Game, card: CardId, target: SlotRef) -> R<bool> {
    let passives: &'static [Passive] = crate::cards::spec_for(g.st.cards[card as usize].def).map_or(&[], |s| s.passives);
    for ps in passives {
        if let Modifier::AttachGuard(a) = &ps.modifier {
            if !attach_guard_allows(g, card, target, a)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Is the Energy `card`, just moved onto `target`, discarded there by its own guard ("if this card is attached to
/// anything other than ..., discard this card": the check the table-state sweep below makes, at the move)?
pub fn attach_guard_discards(g: &mut Game, card: CardId, target: SlotRef) -> R<bool> {
    let passives: &'static [Passive] = crate::cards::spec_for(g.st.cards[card as usize].def).map_or(&[], |s| s.passives);
    for ps in passives {
        if let Modifier::AttachGuard(a) = &ps.modifier {
            let p = target.p as usize;
            if !g.st.slot(p, target.s).cards.contains(card) || g.st.slot_pokemon(p, target.s).is_none() || is_special_energy_blocked(g, p, card, target, false) {
                continue;
            }
            if !slot_pred_m(g, card, target, &a.allow)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn attach_guard(g: &mut Game, me: CardId, e: EffId, a: &AttachGuardSpec) -> R {
    match *g.e(e) {
        Effect::CheckTableState { .. } => {
            for p in 0..2usize {
                for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                    let t = SlotRef::new(p, s);
                    if !g.st.slot(p, s).cards.contains(me) || is_special_energy_blocked(g, p, me, t, false) {
                        continue;
                    }
                    if g.st.slot_pokemon(p, s).is_some() && !slot_pred_m(g, me, t, &a.allow)? {
                        // "Discard this card": it leaves play by its own rule (a LeavePlay of the attached card).
                        let cause = crate::cause::Cause::of_origin(RuleSource::Energy, me, p as u8);
                        crate::engine::knockout::leave_play_cards(g, t, &[me], super::event::RulesZone::Discard, cause, None)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Costs

fn attack_cost(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, c: &AttackCostSpec) -> R {
    let (p, attack) = match *g.e(e) {
        Effect::CheckAttackCost { p, attack, .. } => (p as usize, attack),
        _ => return Ok(()),
    };
    if let Some(i) = c.attack {
        if attack != my_attack(g, me, i) {
            return Ok(());
        }
    }
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if c.side == Side::Owner && at.owner != p || c.side == Side::Opponent && at.owner == p {
        return Ok(());
    }
    let active = SlotRef::new(p, g.st.players[p].active);
    if !slot_pred_m(g, me, active, &c.subject)? || blocked(g, me, origin, at, Some(active)) {
        return Ok(());
    }
    let n = {
        let f = run::Frame::passive(g, me, at.owner, origin);
        if !cond(g, me, &f, &c.guard) {
            return Ok(());
        }
        match &c.change {
            CostChange::Reduce(n) => num(g, me, &f, n),
            _ => 0,
        }
    };
    if let Effect::CheckAttackCost { cost, ignore_colorless, reduction, set_cost, any_reduction, .. } = g.e_mut(e) {
        match &c.change {
            CostChange::AnyOne => *any_reduction = true,
            // Applied once, with the other cost changes, after all handlers ran (D-11, D-12).
            CostChange::Reduce(_) => *reduction = reduction.saturating_add(n.max(0) as u8),
            CostChange::Add(k) => {
                for _ in 0..*k {
                    cost.push(ct::COLORLESS);
                }
            }
            CostChange::Free => cost.clear(),
            CostChange::IgnoreColorless => {
                cost.retain(|t| *t != ct::COLORLESS);
                // ...also the [C] that other effects add (R7F-11, rulings 252, 1552).
                *ignore_colorless = true;
            }
            CostChange::SetCost(set) => {
                let mut v: crate::effects::Cost = SVec::new();
                for t in set.iter() {
                    v.push(*t);
                }
                // A cost that is set is not increased or decreased (R7F-11).
                *set_cost = Some(v);
            }
        }
    }
    Ok(())
}

fn retreat_cost(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, c: &RetreatCostSpec) -> R {
    let p = match *g.e(e) {
        Effect::CheckRetreatCost { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if (c.side == Side::Owner && at.owner != p) || (c.side == Side::Opponent && at.owner == p) {
        return Ok(());
    }
    let mut matched = None;
    let sides: &[usize] = if c.which == RetreatWhich::Mine { &[p] } else { &[p, 1 - p] };
    for q in sides {
        let active = SlotRef::new(*q, g.st.players[*q].active);
        if slot_pred_m(g, me, active, &c.subject)? {
            matched = Some(active);
            break;
        }
    }
    let Some(slot) = matched else { return Ok(()) };
    if blocked(g, me, origin, at, Some(slot)) {
        return Ok(());
    }
    let n = {
        let f = run::Frame::passive(g, me, at.owner, origin);
        if !cond(g, me, &f, &c.guard) {
            return Ok(());
        }
        match &c.change {
            CostChange::Reduce(n) => num(g, me, &f, n),
            _ => 0,
        }
    };
    if let Effect::CheckRetreatCost { cost, no_cost, reduction, .. } = g.e_mut(e) {
        match &c.change {
            CostChange::Reduce(_) => *reduction = reduction.saturating_add(n.max(0) as u8),
            CostChange::Add(k) => {
                for _ in 0..*k {
                    cost.push(ct::COLORLESS);
                }
            }
            CostChange::Free => {
                cost.clear();
                *no_cost = true;
            }
            CostChange::IgnoreColorless => cost.retain(|t| *t != ct::COLORLESS),
            CostChange::SetCost(_) | CostChange::AnyOne => {}
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Special Conditions

/// "Recovers from all Special Conditions": at every table-state check, each Pokémon matching the subject recovers
/// (one RemoveCondition per condition, by the card's effect).
fn recover(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, c: &RecoverSpec) -> R {
    if matches!(*g.e(e), Effect::CheckTableState { .. }) && g.st.any_special_condition() {
        let Some(at) = locate(g, me, origin) else { return Ok(()) };
        for p in 0..2usize {
            for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                let t = SlotRef::new(p, s);
                if g.st.slot(p, s).special_conditions.is_empty() || !slot_pred_m(g, me, t, &c.subject)? || blocked(g, me, origin, at, Some(t)) {
                    continue;
                }
                // The Pokémon recovers: one RemoveCondition per condition, by the card's effect.
                let cause = crate::cause::Cause::of_origin(origin, me, at.owner as u8);
                for x in [SpecialCondition::Poisoned, SpecialCondition::Asleep, SpecialCondition::Burned, SpecialCondition::Confused, SpecialCondition::Paralyzed] {
                    if c.conds.is_empty() || c.conds.contains(&x) {
                        crate::engine::condition::remove(g, t, x, cause)?;
                    }
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Ability locks

fn power_subject(g: &Game, lp: &LockedPowers, power: crate::effects::PowerRef) -> bool {
    if power.index == PROBE_GENERIC {
        return lp.generic_probe;
    }
    let d = &g.st.cdef(power.card).powers[power.index as usize];
    d.power_type == PowerType::Ability as u8
        && !d.exempt_from_ability_lock
        && (!lp.only_knocks_out_self || d.knocks_out_self)
        && lp.exempt_name.map_or(true, |n| d.name != n)
        && !d.use_from_hand
        && !d.use_from_discard
}

/// Is `card`'s Ability locked by this card (the `HANDLE_ABILITY_LOCK` callback)?
fn is_locked(g: &mut Game, me: CardId, card: CardId, l: &AbilityLockSpec) -> R<bool> {
    let owner = locker_owner(g, me, l.locker);
    let Some(owner) = owner else { return Ok(false) };
    let slot = match g.st.locate(card) {
        None => return Ok(pred(g, card, &l.missing)),
        Some(ListRef::Slot(q, s)) => SlotRef::new(q as usize, s),
        Some(_) => return Ok(false),
    };
    if !pred(g, card, &l.card) {
        return Ok(false);
    }
    // A lock source's Ability is turned off only by a lock that wins over it.
    if lock_passives(g, card).next().is_some() && !lock_beats(g, me, card)? {
        return Ok(false);
    }
    match l.probe {
        LockerProbe::Generic => {
            if !slot_pred_m(g, me, slot, &l.slot)? || is_ability_blocked(g, owner, me, None) {
                return Ok(false);
            }
        }
        LockerProbe::OwnPower(i) => {
            if !slot_pred_m(g, me, slot, &l.slot)? {
                return Ok(false);
            }
            let power = crate::effects::PowerRef { card: me, index: i };
            // CAN_APPLY_LOCKER_ABILITY: a real use of the locker's power must not be blocked.
            if g.run_fx(Effect::Power { p: owner as u8, power, card: me, target: None, probe: false }).is_err() {
                return Ok(false);
            }
        }
        LockerProbe::StadiumOnSlot => {
            if is_stadium_effect_blocked(g, slot.p as usize, slot, me) {
                return Ok(false);
            }
            match slot_pred_m(g, me, slot, &l.slot) {
                Ok(v) => {
                    if !v {
                        return Ok(false);
                    }
                }
                // The type check failed: the printed data decides.
                Err(_) => return Ok(pred(g, card, &l.missing)),
            }
        }
    }
    Ok(true)
}

fn ability_lock(g: &mut Game, me: CardId, e: EffId, l: &AbilityLockSpec) -> R {
    match *g.e(e) {
        Effect::CheckPokemonPowers { target, powers, .. } => {
            if is_locked(g, me, target, l)? {
                let mut out = SVec::new();
                for pw in powers.iter() {
                    if !power_subject(g, &l.powers, *pw) {
                        out.push(*pw);
                    }
                }
                if let Effect::CheckPokemonPowers { powers, .. } = g.e_mut(e) {
                    *powers = out;
                }
            }
        }
        Effect::Power { power, card, .. } => {
            if power_subject(g, &l.powers, power) && is_locked(g, me, card, l)? {
                return Err(crate::game::GameError(l.error));
            }
        }
        _ => {}
    }
    Ok(())
}

/// Damp (Psyduck, Golduck): Pokémon in play lose Abilities that require them to Knock Out
/// themselves.
pub const DAMP: AbilityLockSpec = AbilityLockSpec {
    error: "BLOCKED_BY_ABILITY",
    locker: Locker::InPlayEitherSide,
    card: Pred::Any,
    slot: SlotPred::Any,
    missing: Pred::False,
    powers: LockedPowers { generic_probe: false, only_knocks_out_self: true, exempt_name: Some("Damp") },
    probe: LockerProbe::OwnPower(0),
};

// ---------------------------------------------------------------------------
// Prevent

/// Does the prevention make an affected Pokémon recover? A Pokémon that "can't be affected by Special Conditions" /
/// "can't be <condition>", whatever the cause, can't stay affected: one that already is when the protection comes
/// into force (its Ability no longer blocked, a lock gone, the card entering play, the Stadium put into play)
/// recovers (id289: Virizion-EX's Verdant Wind removes the Special Conditions of a Pokémon affected when the
/// protection starts; Festival Grounds and Bubbly Water Energy print it). A prevention that depends on the cause
/// ("effects of attacks used by your opponent's Pokémon": Mist Energy, "Existing effects are not removed") doesn't.
const fn prevention_recovers(p: &PreventSpec) -> bool {
    !p.from.is_never() && p.from.effect_kinds().has(crate::effects::k::GAIN_CONDITION) && !p.from.reads_cause()
}

/// The recovery of [`prevention_recovers`], at the table-state check (where `Modifier::Recover` runs): each Pokémon
/// the declaration protects from a condition it has recovers from it (RemoveCondition by the protecting card).
fn recover_protected(g: &mut Game, me: CardId, origin: RuleSource, p: &PreventSpec) -> R {
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    let cause = crate::cause::Cause::of_origin(origin, me, at.owner as u8);
    for q in 0..2usize {
        for (s, _, _) in for_each_pokemon(g, q, PlayerType::BottomPlayer).iter().copied() {
            let t = SlotRef::new(q, s);
            let conds = g.st.slot(q, s).special_conditions;
            for c in conds.iter().copied() {
                let x = SpecialCondition::from_u8(c);
                let v = crate::engine::condition::condition_view(g, super::event::EventKind::GainCondition, t, x, cause);
                if !p.from.eval(g, me, &v)? || !slot_pred_m(g, me, t, &p.protects)? || blocked(g, me, origin, at, Some(t)) {
                    continue;
                }
                crate::engine::condition::remove(g, t, x, cause)?;
            }
        }
    }
    Ok(())
}

fn prevent(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, p: &PreventSpec) -> R {
    if matches!(*g.e(e), Effect::CheckTableState { .. }) && g.st.any_special_condition() && prevention_recovers(p) {
        return recover_protected(g, me, origin, p);
    }
    // A declaration over events is read by the event's routine (`event_prevented`), not here.
    if matches!(p.what, PreventWhat::None) {
        return Ok(());
    }
    if locate(g, me, origin).is_none() {
        return Ok(());
    }
    match (p.what, *g.e(e)) {
        (PreventWhat::ToolEffects, Effect::Tool { p, card }) => {
            let p = p as usize;
            let mut target: Option<SlotRef> = None;
            for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                if g.st.slot(p, s).tools.contains(card) {
                    target = Some(SlotRef::new(p, s));
                }
            }
            if let Some(t) = target {
                if is_stadium_effect_blocked(g, p, t, NO_CARD) {
                    return Ok(());
                }
            }
            crate::bail!("CANNOT_USE_POWER")
        }
        _ => {}
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// "Prevent all effects of attacks": one `Prevent` over `Cause` (events batch 6)

/// "Prevent all effects of attacks used by your opponent's Pokémon done to ..." (Mist Energy, Rocky Fighting Energy,
/// Unaware, Emperor's Stance, Protective Cover, Repelling Veil): the events caused by the opponent's attacks. It names
/// no kind, so it ranges over every event with an effect and never over Damage (APR C-17: damage is not an effect).
pub const EFFECTS_OF_OPP_ATTACKS: super::event::EventPred = super::event::EventPred::Cause(super::event::CausePred::All(&[super::event::CausePred::By(Who::Opp), super::event::CausePred::Kind(crate::cause::CauseKind::Attack)]));

/// Hide 'n' Sneak's "prevent all effects of your opponent's Pokémon's attacks and Abilities done to this Pokémon".
pub const EFFECTS_OF_OPP_ATTACKS_AND_ABILITIES: super::event::EventPred = super::event::EventPred::Cause(super::event::CausePred::All(&[
    super::event::CausePred::By(Who::Opp),
    super::event::CausePred::Any(&[super::event::CausePred::Kind(crate::cause::CauseKind::Attack), super::event::CausePred::Kind(crate::cause::CauseKind::Ability)]),
]));

/// "Damage from and effects of": the Damage event or any event with an effect (`All[DAMAGE_OR_EFFECTS, Cause(..)]`
/// restricts both to the cause: Rabsca, Acerola's Mischief, Milotic ex).
pub const DAMAGE_OR_EFFECTS: super::event::EventPred = super::event::EventPred::Any(&[super::event::EventPred::Kind(super::event::EventKind::Damage), super::event::EventPred::ALWAYS]);

/// Hide 'n' Sneak (Shuppet, Banette, Sinistcha, Poltchageist): no Bench condition (the printed text has none; id2426
/// has it work on an Active Pokémon).
pub const HIDE_N_SNEAK: PreventSpec = PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EFFECTS_OF_OPP_ATTACKS_AND_ABILITIES);

/// "During your opponent's next turn, prevent all effects of attacks done to this Pokémon" (an attack's lasting effect on
/// its own Pokémon: `Lasting::PreventAttackEffects`), stored on the Pokémon (`Slot::lasting_prevents`).
pub static LASTING_PREVENT_EFFECTS: PreventSpec = PreventSpec::on(SlotPred::Any, EFFECTS_OF_OPP_ATTACKS);

/// "Is damaged by an attack" (no owner named: any attack's damage, its own player's included; Fezandipiti).
pub const DAMAGE_BY_ATTACKS: super::event::EventPred = super::event::EventPred::All(&[
    super::event::EventPred::Kind(super::event::EventKind::Damage),
    super::event::EventPred::Cause(super::event::CausePred::Kind(crate::cause::CauseKind::Attack)),
]);

/// "Prevent all damage done to ... by attacks from your opponent's Pokémon": the Damage event caused by the opponent's
/// attacks.
pub const DAMAGE_BY_OPP_ATTACKS: super::event::EventPred = super::event::EventPred::All(&[
    super::event::EventPred::Kind(super::event::EventKind::Damage),
    super::event::EventPred::Cause(super::event::CausePred::All(&[super::event::CausePred::By(Who::Opp), super::event::CausePred::Kind(crate::cause::CauseKind::Attack)])),
]);

/// The Tera rule ("As long as this Pokémon is on your Bench, prevent all damage done to this Pokémon by attacks (both
/// yours and your opponent's)"): a card rule, not an Ability.
pub const TERA_RULE: PreventSpec = PreventSpec::on(
    SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon, SlotPred::IsBench]),
    super::event::EventPred::Kind(super::event::EventKind::Damage),
);

/// The lasting "during your opponent's next turn, prevent all damage done to this Pokémon by attacks (from <these>
/// Pokémon)" an attack leaves on its Pokémon (`DamageSource::spec`), stored with it (`Slot::lasting_prevents`). The
/// attacking Pokémon is read where it is now (`CausePred::Pokemon`).
pub static LASTING_PREVENT_DAMAGE: PreventSpec = PreventSpec::on(SlotPred::Any, DAMAGE_BY_OPP_ATTACKS);
pub static LASTING_PREVENT_DAMAGE_FROM_BASIC: PreventSpec = PreventSpec::on(SlotPred::Any, super::event::EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, super::event::EventPred::Cause(super::event::CausePred::Pokemon(SlotPred::Basic))]));
pub static LASTING_PREVENT_DAMAGE_FROM_EVOLUTION: PreventSpec =
    PreventSpec::on(SlotPred::Any, super::event::EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, super::event::EventPred::Cause(super::event::CausePred::Pokemon(SlotPred::Not(&SlotPred::Basic)))]));
/// "Pokémon that have an Ability": a Pokémon whose Abilities are locked has none.
pub static LASTING_PREVENT_DAMAGE_FROM_ABILITY: PreventSpec =
    PreventSpec::on(SlotPred::Any, super::event::EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, super::event::EventPred::Cause(super::event::CausePred::Pokemon(SlotPred::HasAbility))]));
/// "Basic Pokémon that aren't [C]" (printed types, as the Pokémon's card says).
pub static LASTING_PREVENT_DAMAGE_FROM_BASIC_NON_COLORLESS: PreventSpec = PreventSpec::on(
    SlotPred::Any,
    super::event::EventPred::All(&[
        DAMAGE_BY_OPP_ATTACKS,
        super::event::EventPred::Cause(super::event::CausePred::Pokemon(SlotPred::All(&[
            SlotPred::Basic,
            SlotPred::Top(Pred::OneOf(&[
                Pred::PokemonType(crate::types::ct::GRASS),
                Pred::PokemonType(crate::types::ct::FIRE),
                Pred::PokemonType(crate::types::ct::WATER),
                Pred::PokemonType(crate::types::ct::LIGHTNING),
                Pred::PokemonType(crate::types::ct::PSYCHIC),
                Pred::PokemonType(crate::types::ct::FIGHTING),
                Pred::PokemonType(crate::types::ct::DARK),
                Pred::PokemonType(crate::types::ct::METAL),
                Pred::PokemonType(crate::types::ct::FAIRY),
                Pred::PokemonType(crate::types::ct::DRAGON),
            ])),
        ]))),
    ]),
);

/// Every lasting `Prevent` an attack can leave on a Pokémon (`Slot::lasting_prevents` stores an index into it).
pub static LASTING_PREVENTS: [&PreventSpec; 6] = [
    &LASTING_PREVENT_EFFECTS,
    &LASTING_PREVENT_DAMAGE,
    &LASTING_PREVENT_DAMAGE_FROM_BASIC,
    &LASTING_PREVENT_DAMAGE_FROM_EVOLUTION,
    &LASTING_PREVENT_DAMAGE_FROM_ABILITY,
    &LASTING_PREVENT_DAMAGE_FROM_BASIC_NON_COLORLESS,
];

/// The index of a lasting `Prevent` in [`LASTING_PREVENTS`].
pub fn lasting_index(spec: &'static PreventSpec) -> u8 {
    LASTING_PREVENTS.iter().position(|s| std::ptr::eq(*s, spec)).expect("a lasting Prevent is in LASTING_PREVENTS") as u8
}

/// Does a `Prevent` over `from` range over events of `kind` (the mask test the reader makes before evaluating it)? An
/// event kind with no effect is in range of a declaration that ranges over every effect.
pub fn ranges(from: &super::event::EventPred, kind: super::event::EventKind) -> bool {
    let m = from.prevent_kinds();
    match kind.effect_kind() {
        Some(x) => m.has(x),
        None => and(m, super::event::EFFECT_EVENT_KINDS) == super::event::EFFECT_EVENT_KINDS,
    }
}

// ---------------------------------------------------------------------------
// Prizes, evolution, attacks

fn prize_adjust(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &PrizeAdjustSpec) -> R {
    let (p, target, ko_by) = match *g.e(e) {
        Effect::KnockOut { p, target, ko_by, .. } => (p as usize, target, ko_by),
        _ => return Ok(()),
    };
    if !slot_pred_m(g, me, target, &d.subject)? {
        return Ok(());
    }
    if let Some(name) = d.by_own_attack {
        // The Knock Out happens in the attack phase of the opponent of the Knocked Out Pokémon's owner.
        let attacker = 1 - p;
        let pl = &g.st.players[p];
        let defending = target.p as usize == p && (pl.active == target.s || pl.bench.iter().any(|b| *b == target.s));
        if !defending || g.st.phase != GamePhase::Attack || g.st.active_player as usize != attacker {
            return Ok(());
        }
        match g.st.player_last_attack[attacker] {
            Some((a, src)) if src == me && crate::engine::attack::attack_def(g, a).name == name => {}
            _ => return Ok(()),
        }
    }
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if d.owner_only && at.owner != p {
        return Ok(());
    }
    if blocked(g, me, origin, at, Some(target)) {
        return Ok(());
    }
    // "Knocked Out by damage from an attack": the KnockOut event's `ko_by`.
    let by_damage = ko_by == super::event::KoBy::AttackDamage;
    if d.by_attack_damage && !by_damage {
        return Ok(());
    }
    if by_damage {
        if let Some((_, Some(src))) = g.knocked_out_by_attack_damage(p, target) {
            if !slot_pred_m(g, me, src, &d.attacker)? {
                return Ok(());
            }
        }
    }
    if !guard_ok(g, me, origin, at.owner, &d.guard) {
        return Ok(());
    }
    if let Some(name) = d.nonstacking {
        let id = crate::markers::intern(name);
        if g.st.slot(target.p as usize, target.s).marker.has(id) {
            return Ok(());
        }
        g.st.players[target.p as usize].slots[target.s as usize].marker.add(id, me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    }
    if let Effect::KnockOut { prize_count, .. } = g.e_mut(e) {
        *prize_count += d.delta;
    }
    Ok(())
}

fn grant_attacks(g: &mut Game, me: CardId, e: EffId, origin: RuleSource) -> R {
    let p = match *g.e(e) {
        Effect::CheckPokemonAttacks { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if at.owner != p || blocked(g, me, origin, at, None) {
        return Ok(());
    }
    let mut add: SVec<AttackRef, 32> = SVec::new();
    let active = g.st.players[p].active;
    if let Some(top) = g.st.slot_pokemon(p, active) {
        if g.st.cdef(top).stage != Stage::Basic as u8 {
            for c in g.st.slot(p, active).cards.iter() {
                let d = g.st.cdef(c);
                if d.is_pokemon() && c != top {
                    for i in 0..d.attacks.len() {
                        add.push(AttackRef { card: c, index: i as u8 });
                    }
                }
            }
        }
    }
    if let Effect::CheckPokemonAttacks { attacks, .. } = g.e_mut(e) {
        for a in add.iter() {
            attacks.push(*a);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Blocked plays

// ---------------------------------------------------------------------------
// Locks (`Modifier::BlockUse`, the lasting ones), declared as data
//
// `event_locked` is the one query: the event's routine and legality read the same declarations, so they can't drift.

/// The lock that forbids an event, if any: the error code of the first one. The locked player is the event's
/// actor (`EventView::actor`, the `Cause` player: who does the play, the attach, ...), not the owner of the
/// card (id25, id230, id959, id20). Asked by the event's routine before the event, and by legality (the same
/// declarations): the locks over events (`LockDecl::forbids`) in play, in propagation order, then the lasting
/// ones on the actor.
pub fn event_locked(g: &mut Game, v: &super::event::EventView) -> R<Option<&'static str>> {
    // A Pokémon event or an Attach without a card is one whose card isn't chosen yet (legality asks it only with
    // one). A CoinFlip has no card (a coin isn't a card): `EventPred::Card(..)` is false on it (`v.card` stays None;
    // test `coin_flip_has_no_card`); the probe below only orders the lock cards, whatever its card.
    if v.card.is_none() && v.kind != super::event::EventKind::CoinFlip {
        return Ok(None);
    }
    let p = v.actor() as usize;
    if !may_lock_event(g, p, v.kind) {
        return Ok(None);
    }
    if let (Some(marker), Some(kind)) = (lock_marker(v.kind), v.kind.effect_kind()) {
        if g.kinds_present.has(marker) {
            let probe = Effect::EndTurn { p: p as u8 };
            let order = g.propagation_order(&probe, kind);
            for c in order.iter().copied() {
                if let Some(code) = event_locked_by(g, c, v)? {
                    return Ok(Some(code));
                }
            }
        }
    }
    lasting_event_locked(g, v)
}

/// The declaration marker a lock over events of `kind` sets in its card's mask (`block_kinds`), `None` for a
/// kind no effect carries yet (a lock over it would never be consulted: `block_kinds` sets nothing for it).
/// Exhaustive on purpose: the batch that makes a kind an event gives it a marker here, and
/// `event_lock_marker_tests` fails until it does.
#[inline]
pub(crate) const fn lock_marker(kind: super::event::EventKind) -> Option<u32> {
    use super::event::EventKind as E;
    match kind {
        E::EnterPlay | E::Evolve | E::Devolve | E::Swap => Some(crate::effects::k::DECLARES_EVENT_LOCK),
        E::Attach | E::MoveEnergy | E::MoveTool => Some(crate::effects::k::DECLARES_ATTACH_LOCK),
        E::GainCondition | E::RemoveCondition => Some(crate::effects::k::DECLARES_CONDITION_LOCK),
        E::RemoveCounters => Some(crate::effects::k::DECLARES_HEAL_LOCK),
        E::CoinFlip => Some(crate::effects::k::DECLARES_COIN_LOCK),
        E::ChangeActive => Some(crate::effects::k::DECLARES_ACTIVE_LOCK),
        E::PlaceCounters | E::MoveCounters => Some(crate::effects::k::DECLARES_COUNTER_LOCK),
        E::Discard | E::Draw | E::PutIntoHand | E::PutIntoDeck => Some(crate::effects::k::DECLARES_CARD_LOCK),
        E::PlayTrainer => Some(crate::effects::k::DECLARES_PLAY_LOCK),
        E::Damage
        | E::ApplyEffect
        | E::KnockOut
        | E::TakePrizes
        | E::LeavePlay
        | E::Shuffle
        | E::Look
        | E::Reveal
        | E::StateCheck
        | E::GameEnd
        | E::Mulligan
        | E::SetPrizes
        | E::BeginTurn
        | E::EndTurn
        | E::Checkup
        | E::UseAttack
        | E::UseAbility
        | E::UseStadium
        | E::Retreat => None,
    }
}

/// Can a lock forbid an event of `kind` of player `p` at all (a plain read: some card of the game declares a
/// lock over events of its family, or an attack left a lock over events on `p`)? When it can't,
/// [`event_locked`] answers `None` without a walk, so legality asks it before making its scratch game.
#[inline]
pub fn may_lock_event(g: &Game, p: usize, kind: super::event::EventKind) -> bool {
    lock_marker(kind).map_or(false, |m| g.kinds_present.has(m)) || g.st.players[p].lasting_locks.iter().flatten().any(|l| !l.decl.forbids.is_never())
}

/// The marker a `Prevent` over events of `kind` sets in its card's mask (`prevent_kinds`) and that
/// [`event_prevented`] reads; `None` for a kind whose routine doesn't ask the reader (a prevention over it would
/// never be consulted: `prevent_marker_tests` checks that no card declares one). Exhaustive, like `lock_marker`.
#[inline]
pub(crate) const fn prevent_marker(kind: super::event::EventKind) -> Option<u32> {
    use super::event::EventKind as E;
    match kind {
        E::GainCondition | E::RemoveCondition => Some(crate::effects::k::DECLARES_CONDITION_PREVENT),
        E::RemoveCounters => Some(crate::effects::k::DECLARES_HEAL_PREVENT),
        E::CoinFlip => Some(crate::effects::k::DECLARES_COIN_PREVENT),
        E::ChangeActive => Some(crate::effects::k::DECLARES_ACTIVE_PREVENT),
        E::PlaceCounters | E::MoveCounters => Some(crate::effects::k::DECLARES_COUNTER_PREVENT),
        E::Damage => Some(crate::effects::k::DECLARES_DAMAGE_PREVENT),
        E::KnockOut => Some(crate::effects::k::DECLARES_KO_PREVENT),
        E::Attach | E::MoveEnergy | E::MoveTool => Some(crate::effects::k::DECLARES_ATTACH_PREVENT),
        E::Evolve | E::Devolve | E::Swap => Some(crate::effects::k::DECLARES_POKEMON_PREVENT),
        E::ApplyEffect => Some(crate::effects::k::DECLARES_APPLY_PREVENT),
        E::LeavePlay => Some(crate::effects::k::DECLARES_LEAVE_PREVENT),
        E::EnterPlay
        | E::PlayTrainer
        | E::TakePrizes
        | E::Discard
        | E::Draw
        | E::PutIntoHand
        | E::PutIntoDeck
        | E::Shuffle
        | E::Look
        | E::Reveal
        | E::StateCheck
        | E::GameEnd
        | E::Mulligan
        | E::SetPrizes
        | E::BeginTurn
        | E::EndTurn
        | E::Checkup
        | E::UseAttack
        | E::UseAbility
        | E::UseStadium
        | E::Retreat => None,
    }
}

/// Is the event prevented (a `Prevent` declaration whose `from` matches it protects its spot)? The events
/// design's Prevent reader: the event's routine asks it (through `derived::event_prevented`) after the locks, only
/// in a game where a card declares a prevention over the event's family (`prevent_marker`), so a game without one
/// pays a mask test. The sources are walked in the propagation order of the event's kind.
pub fn event_prevented(g: &mut Game, v: &super::event::EventView) -> R<bool> {
    let (Some(marker), Some(kind)) = (prevent_marker(v.kind), v.kind.effect_kind()) else { return Ok(false) };
    if !g.kinds_present.has(marker) {
        return Ok(false);
    }
    let probe = Effect::EndTurn { p: v.owner };
    let order = g.propagation_order(&probe, kind);
    for c in order.iter().copied() {
        if prevented_by(g, c, v)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The `Prevent` declarations an attack left on the event's Pokémon ("during your opponent's next turn, prevent all
/// damage done to / effects of attacks done to this Pokémon": `Slot::lasting_prevents`, in force once no longer
/// pending): one matches the event. Each is evaluated for the card whose attack left it. A plain read of the slot when
/// it holds none.
pub fn lasting_prevented(g: &mut Game, v: &super::event::EventView) -> R<bool> {
    let Some(t) = v.slot else { return Ok(false) };
    let n = g.st.slot(t.p as usize, t.s).lasting_prevents.len();
    for i in 0..n {
        let l = g.st.slot(t.p as usize, t.s).lasting_prevents.as_slice()[i];
        let spec = l.spec();
        if l.pending || spec.coin || !ranges(&spec.from, v.kind) {
            continue;
        }
        if spec.from.eval(g, l.source, v)? && slot_pred_m(g, l.source, t, &spec.protects)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// [`event_prevented`] for one source `me`: one of its `Prevent` declarations matches the event, protects the
/// event's spot, and the card is in place for its origin and not blocked there.
fn prevented_by(g: &mut Game, me: CardId, v: &super::event::EventView) -> R<bool> {
    let passives: &'static [Passive] = crate::cards::spec_for(g.st.cards[me as usize].def).map_or(&[], |s| s.passives);
    for ps in passives {
        let Modifier::Prevent(p) = &ps.modifier else { continue };
        // The kinds it ranges over first (a mask test): "prevent all effects of attacks" names no kind and never
        // prevents Damage (APR C-17).
        if p.from.is_never() || p.coin || !ranges(&p.from, v.kind) {
            continue;
        }
        if !p.from.eval(g, me, v)? {
            continue;
        }
        let Some(at) = locate(g, me, ps.origin) else { continue };
        let protected = match v.slot {
            Some(s) => slot_pred_m(g, me, s, &p.protects)?,
            None => matches!(p.protects, SlotPred::Any),
        };
        if protected && !blocked(g, me, ps.origin, at, v.slot) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The coin-flip preventions over the event ("flip a coin; if heads, prevent that damage": `PreventSpec::coin`), asked
/// after the others found none (user decision D8): each matching one in place for its origin and not blocked makes its
/// card's owner flip (a CoinFlip by its origin); heads prevents the event.
pub fn coin_prevented(g: &mut Game, v: &super::event::EventView) -> R<bool> {
    let (Some(marker), Some(kind)) = (prevent_marker(v.kind), v.kind.effect_kind()) else { return Ok(false) };
    if !g.kinds_present.has(marker) {
        return Ok(false);
    }
    let probe = Effect::EndTurn { p: v.owner };
    let order = g.propagation_order(&probe, kind);
    let mut heads = false;
    for me in order.iter().copied() {
        let passives: &'static [Passive] = crate::cards::spec_for(g.st.cards[me as usize].def).map_or(&[], |s| s.passives);
        for ps in passives {
            let Modifier::Prevent(p) = &ps.modifier else { continue };
            if !p.coin || !ranges(&p.from, v.kind) {
                continue;
            }
            let Some(at) = locate(g, me, ps.origin) else { continue };
            if !p.from.eval(g, me, v)? {
                continue;
            }
            let protected = match v.slot {
                Some(s) => slot_pred_m(g, me, s, &p.protects)?,
                None => matches!(p.protects, SlotPred::Any),
            };
            if !protected || blocked(g, me, ps.origin, at, v.slot) {
                continue;
            }
            let cause = crate::cause::Cause::of_origin(ps.origin, me, at.owner as u8);
            let (c, _) = g.run_fx(Effect::CoinFlipRequest { p: at.owner as u8, callback: None, result: None, skip_reflip_stadium: false, skip_reflip_tool: false, cause })?;
            if let Effect::CoinFlipRequest { result: Some(true), .. } = c {
                heads = true;
            }
        }
    }
    Ok(heads)
}

/// [`event_locked`] for one in-play lock source `me`.
pub(crate) fn event_locked_by(g: &mut Game, me: CardId, v: &super::event::EventView) -> R<Option<&'static str>> {
    let passives: &'static [Passive] = crate::cards::spec_for(g.st.cards[me as usize].def).map_or(&[], |s| s.passives);
    for ps in passives {
        if let Modifier::BlockUse(b) = &ps.modifier {
            if let Some(code) = event_lock_blocks(g, me, ps.origin, b, v)? {
                return Ok(Some(code));
            }
        }
    }
    Ok(None)
}

/// [`event_locked`] for the locks over events an attack left on the event's actor (the locked player).
pub fn lasting_event_locked(g: &mut Game, v: &super::event::EventView) -> R<Option<&'static str>> {
    let p = v.actor() as usize;
    for i in 0..g.st.players[p].lasting_locks.len() {
        let Some(l) = g.st.players[p].lasting_locks[i] else { continue };
        if l.decl.coin == CoinGate::No && !l.decl.forbids.is_never() && l.decl.forbids.eval(g, l.source, v)? {
            return Ok(Some(l.decl.error));
        }
    }
    Ok(None)
}

/// A coin-gated lock an attack left on the event's actor that the event matches (Seismitoad's Quaking Fist: "whenever
/// they try to use a Trainer card from their hand, they flip a coin"): the cause of the coin (the lock's source's
/// attack, by its owner), or `None` when no such lock applies.
pub fn coin_gate(g: &mut Game, v: &super::event::EventView) -> R<Option<crate::cause::Cause>> {
    let p = v.actor() as usize;
    for i in 0..g.st.players[p].lasting_locks.len() {
        let Some(l) = g.st.players[p].lasting_locks[i] else { continue };
        if l.decl.coin != CoinGate::No && l.decl.forbids.eval(g, l.source, v)? {
            return Ok(Some(crate::cause::Cause::new(crate::cause::CauseKind::Attack, Some(l.source), g.st.owner(l.source) as u8)));
        }
    }
    Ok(None)
}

/// The cause of the coin-gated lock on player `p` (its tails' discard), if one is there.
pub fn coin_gate_cause(g: &Game, p: usize) -> Option<crate::cause::Cause> {
    g.st.players[p].lasting_locks.iter().flatten().find(|l| l.decl.coin != CoinGate::No).map(|l| crate::cause::Cause::new(crate::cause::CauseKind::Attack, Some(l.source), g.st.owner(l.source) as u8))
}

/// Does the in-play lock `b` of `me` forbid the event (`LockDecl::forbids`)?
pub(crate) fn event_lock_blocks(g: &mut Game, me: CardId, origin: RuleSource, b: &BlockUseSpec, v: &super::event::EventView) -> R<Option<&'static str>> {
    if b.lock.forbids.is_never() {
        return Ok(None);
    }
    let Some(at) = locate(g, me, origin) else { return Ok(None) };
    // The locked player is the actor (`EventView::actor`).
    let p = v.actor() as usize;
    let binds = match b.binds {
        Binds::Opponent => p == 1 - at.owner,
        Binds::Owner => p == at.owner,
        Binds::Both => true,
    };
    if !binds || !crate::engine::enter::while_ok(g, me, at, b.while_, v.card) {
        return Ok(None);
    }
    if !b.lock.forbids.eval(g, me, v)? {
        return Ok(None);
    }
    if b.ability && !source_ability_on(g, at.owner, me) {
        return Ok(None);
    }
    Ok(Some(b.lock.error))
}

/// Does the lock source `me` have its Ability: the stored lock state when it can say, else the probe
/// (`PTCG_VERIFY_LEGAL=1` checks the two against each other).
fn source_ability_on(g: &mut Game, owner: usize, me: CardId) -> bool {
    let off = ability_off(g, me);
    match off {
        Some(off) if !verify_ability_off() => !off,
        _ => {
            let blocked = is_ability_blocked(g, owner, me, None);
            if let Some(off) = off {
                assert_eq!(off, blocked, "ability_off differs from the lock probe for card {me}");
            }
            !blocked
        }
    }
}

fn verify_ability_off() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("PTCG_VERIFY_LEGAL").map_or(false, |v| v == "1"))
}

/// Is the Pokémon card's Ability off (turned off by a lock, the generic Ability probe)? Answered from the
/// stored lock state (`CardInst::lock_stamp` of every lock source, as `lock_sync` keeps it) and the lock
/// declarations, without running an effect; `None` when it can't be sure:
/// * a copied attack is being run (`deleg`) for another card, or a lock sync is running;
/// * a lock source that is in place has no stamp: it is off by another lock, or the stamps are not yet
///   synced;
/// * a lock that covers a lock source (whether it wins is `lock_beats`, a checked read);
/// * Damp (only Abilities that Knock Out their user are removed, so a generic answer isn't defined) when the
///   card has such an Ability;
/// * a Pokémon type that is a checked read while a card changes types.
///
/// A card that isn't a Pokémon in play (the hand, the discard pile) has its printed Ability: no lock reaches it.
pub fn ability_off(g: &Game, card: CardId) -> Option<bool> {
    use crate::effects::k;
    if let Some(d) = g.deleg {
        return if d.attacks && d.copycat == card { Some(true) } else { None };
    }
    if g.lock_syncing {
        return None;
    }
    if !g.kinds_present.has(k::CHECK_POKEMON_POWERS) {
        return Some(false);
    }
    let slot = match g.st.locate(card) {
        Some(ListRef::Slot(q, s)) => SlotRef::new(q as usize, s),
        Some(_) => return Some(false),
        None => return None,
    };
    let mut sources: SVec<CardId, 24> = SVec::new();
    for q in 0..2usize {
        for s in g.st.players[q].in_play().iter() {
            if let Some(c) = g.st.slot_pokemon(q, *s) {
                sources.push(c);
            }
        }
    }
    if let Some(c) = g.st.stadium_card() {
        sources.push(c);
    }
    let card_is_lock = lock_passives(g, card).next().is_some();
    let mut unsure = false;
    for src in sources.iter().copied() {
        for pas in lock_passives(g, src) {
            let Some(owner) = lock_position(g, src, pas) else { continue };
            if g.st.cards[src as usize].lock_stamp == 0 {
                unsure = true;
                continue;
            }
            match lock_covers_generic(g, src, pas, owner, card, slot) {
                Some(true) if card_is_lock => unsure = true,
                Some(true) => return Some(true),
                Some(false) => {}
                None => unsure = true,
            }
        }
    }
    if unsure {
        None
    } else {
        Some(false)
    }
}

/// Would lock `pas` of `src` (in place, holding), take the Ability of `card` in `slot` away from the generic
/// probe? `None` when that needs a checked read.
fn lock_covers_generic(g: &Game, src: CardId, pas: &Passive, owner: usize, card: CardId, slot: SlotRef) -> Option<bool> {
    match &pas.modifier {
        Modifier::ActiveLock(ActiveLock::MidnightFluttering) => {
            // Hide 'n' Sneak takes precedence over Midnight Fluttering.
            Some(slot == SlotRef::new(1 - owner, g.st.players[1 - owner].active) && !g.st.cdef(card).powers.iter().any(|pw| pw.name == "Hide 'n' Sneak"))
        }
        Modifier::ActiveLock(ActiveLock::Initialization) => {
            let d = g.st.cdef(card);
            Some(!d.has_tag(tag::FUTURE) && d.has_rule_box())
        }
        Modifier::AbilityLock(l) => {
            if !l.powers.generic_probe {
                // Damp: decided per Ability.
                let any = (0..g.st.cdef(card).powers.len()).any(|i| power_subject(g, &l.powers, crate::effects::PowerRef { card, index: i as u8 }));
                return if any { None } else { Some(false) };
            }
            if !pred(g, card, &l.card) {
                return Some(false);
            }
            if matches!(l.probe, LockerProbe::StadiumOnSlot) && g.st.players.iter().any(|pl| pl.stadium_and_tool_have_no_effect_turns_remaining > 0) {
                return Some(false);
            }
            slot_pred_ro(g, src, slot, &l.slot)
        }
        _ => Some(false),
    }
}

/// A slot predicate read without running effects; the Pokémon type reads the printed type while no card
/// changes types.
fn slot_pred_ro(g: &Game, me: CardId, s: SlotRef, sp: &SlotPred) -> Option<bool> {
    match slot_pred(g, me, s, sp) {
        Some(v) => Some(v),
        None => match sp {
            SlotPred::TypeIs(t) if !g.kinds_present.has(crate::effects::k::CHECK_POKEMON_TYPE) => Some(crate::engine::game_effect::pokemon_types(g, s).contains(t)),
            _ => None,
        },
    }
}

// ---------------------------------------------------------------------------
// Survive on 10 (S3 agent 3)

fn survive_on_ten(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, spec: &SurviveOnTenSpec) -> R {
    let (t, damage) = match *g.e(e) {
        Effect::PutDamage { b, damage, .. } => (b.target, damage),
        _ => return Ok(()),
    };
    if spec.kind == SurviveKind::ToolIfFullHp {
        return survive_on_ten_tool(g, me, e, origin);
    }
    let owner = t.p as usize;
    if !g.st.slot(owner, t.s).cards.contains(me) {
        return Ok(());
    }
    let at = Located { owner, held: Some(t) };
    if blocked(g, me, origin, at, Some(t)) {
        return Ok(());
    }
    match spec.kind {
        SurviveKind::ToolIfFullHp => {}
        SurviveKind::OnCoin => {
            // Only the Pokémon on top has the Ability.
            if g.st.slot_pokemon(owner, t.s) != Some(me) {
                return Ok(());
            }
            survive_on_ten_on_coin_flip(g, e, owner, crate::cause::Cause::of_origin(origin, me, owner as u8))?
        }
        SurviveKind::IfFullHp => {
            if g.st.slot(owner, t.s).damage != 0 {
                return Ok(());
            }
            let hp = crate::derived::hp(g, owner, t.s)?;
            if damage >= hp {
                if let Effect::PutDamage { survive_on_ten_hp, .. } = g.e_mut(e) {
                    *survive_on_ten_hp = true;
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Checkup damage (S3 agent 3)

fn checkup_damage(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, c: &CheckupDamageSpec) -> R {
    let p = match *g.e(e) {
        Effect::BetweenTurns { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if c.opponent_only && at.owner == p {
        return Ok(());
    }
    if let Some(held) = at.held {
        if !slot_pred_m(g, me, held, &c.holder)? {
            return Ok(());
        }
    }
    let victim = SlotRef::new(p, g.st.players[p].active);
    if blocked(g, me, origin, at, Some(victim)) || !slot_pred_m(g, me, victim, &c.victim)? {
        return Ok(());
    }
    if c.burn {
        if let Effect::BetweenTurns { burn_damage, .. } = g.e_mut(e) {
            *burn_damage += c.amount;
        }
    } else if let Effect::BetweenTurns { poison_damage, .. } = g.e_mut(e) {
        *poison_damage += c.amount;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Next-turn attack bonus and first-turn attacks (S3 agent 3)

fn next_turn_bonus(g: &mut Game, me: CardId, e: EffId, b: &NextTurnBonusSpec) -> R {
    let (attack, source) = match *g.e(e) {
        Effect::Attack { attack, source, .. } => (attack, source),
        _ => return Ok(()),
    };
    if g.st.slot_pokemon(source.p as usize, source.s) != Some(me) {
        return Ok(());
    }
    let full_name = g.st.cdef(me).full_name;
    let attack_name = g.st.cdef(attack.card).attacks[attack.idx()].name;
    let slot = &g.st.players[source.p as usize].slots[source.s as usize];
    let armed = match slot.next_turn_attack_damage_bonus {
        Some(a) if a.source_card_name == full_name && (a.attack_name == "*" || a.attack_name == attack_name) => a.bonus_damage,
        _ => 0,
    };
    if armed != 0 {
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += armed;
        }
    }
    if attack_name != b.attack {
        return Ok(());
    }
    g.st.players[source.p as usize].slots[source.s as usize].next_turn_attack_damage_bonus_pending =
        Some(crate::state::NextTurnAttackDamageBonus { attack_name: b.attack, bonus_damage: b.bonus, source_card_name: full_name });
    Ok(())
}

fn attack_flags(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, a: &AttackFlagsSpec) -> R {
    if let Some(attacker) = &a.shred {
        return shred_flags(g, me, e, origin, attacker);
    }
    let Effect::UseAttack { p, attack, .. } = *g.e(e) else { return Ok(()) };
    let p = p as usize;
    if !first_turn_grants(g, me, origin, a, p) {
        return Ok(());
    }
    // A copy-attack clone carries its own flag (nothing reads it).
    if !attack.is_clone() {
        g.st.cards[attack.card as usize].attack_first_turn |= 1u8 << attack.idx();
    }
    Ok(())
}

/// Does `me` (an Ability on the attacking player's Active Pokémon) let `p`'s attacks be used on the first turn?
fn first_turn_grants(g: &mut Game, me: CardId, origin: RuleSource, a: &AttackFlagsSpec, p: usize) -> bool {
    let active = g.st.players[p].active;
    if !a.first_turn || !g.st.slot(p, active).cards.contains(me) || g.st.turn != 1 {
        return false;
    }
    let at = Located { owner: p, held: None };
    !blocked(g, me, origin, at, None)
}

/// Legality: will an Ability on `p`'s Active Pokémon let the attack be used on the first turn (the flag
/// `attack_flags` writes when the `UseAttack` effect reaches it)?
pub fn grants_first_turn_attack(g: &mut Game, p: usize) -> bool {
    if g.st.turn != 1 || !g.kinds_present.has(crate::effects::k::USE_ATTACK) {
        return false;
    }
    let active = g.st.players[p].active;
    let mut cards: SVec<CardId, 8> = SVec::new();
    for c in g.st.slot(p, active).cards.iter() {
        cards.push(c);
    }
    for me in cards.iter().copied() {
        let passives: &'static [Passive] = crate::cards::spec_for(g.st.cards[me as usize].def).map_or(&[], |s| s.passives);
        for ps in passives {
            if let Modifier::AttackFlags(a) = &ps.modifier {
                if a.shred.is_none() && first_turn_grants(g, me, ps.origin, a, p) {
                    return true;
                }
            }
        }
    }
    false
}

fn bench_attacks(g: &mut Game, me: CardId, e: EffId, origin: RuleSource) -> R {
    let Effect::CheckPokemonAttacks { p, .. } = *g.e(e) else { return Ok(()) };
    let p = p as usize;
    if g.st.active_pokemon(p) != Some(me) || blocked(g, me, origin, Located { owner: p, held: None }, None) {
        return Ok(());
    }
    let mut add: SVec<AttackRef, 32> = SVec::new();
    let bench: Vec<crate::state::SlotId> = g.st.players[p].bench.iter().copied().collect();
    for b in bench {
        if let Some(c) = g.st.slot_pokemon(p, b) {
            for i in 0..g.st.cdef(c).attacks.len() {
                add.push(AttackRef { card: c, index: i as u8 });
            }
        }
    }
    if let Effect::CheckPokemonAttacks { attacks, copied, .. } = g.e_mut(e) {
        for a in add.iter() {
            attacks.push(*a);
            copied.push(*a);
        }
    }
    Ok(())
}


// Active-Spot Ability locks (Midnight Fluttering, Initialization)

// ---------------------------------------------------------------------------
// Precedence between locks (docs/rulings/RULES.md, 2026-10-08)
//
// A lock source is a card with an Ability-lock passive (Flutter Mane, Iron Thorns ex, Gastrodon, Psyduck,
// Team Rocket's Watchtower, ...). When lock A turns off the Ability of lock source B:
// * one-way (B doesn't turn A's Ability off): A wins whatever the order;
// * mutual: the lock that took hold first wins, decided by the take-hold stamps.
// A lock takes hold when its conditions are first met while its source has its Ability; its stamp
// (`CardInst::lock_stamp`) is the next number then, and is cleared when it goes off. Locks taking hold
// together are stamped with the turn player's first. `lock_sync` keeps the stamps up to date.

/// The Ability-lock passives of a card.
fn lock_passives(g: &Game, c: CardId) -> impl Iterator<Item = &'static Passive> {
    let passives: &'static [Passive] = crate::cards::spec_for(g.st.cards[c as usize].def).map_or(&[], |s| s.passives);
    passives.iter().filter(|p| matches!(p.modifier, Modifier::ActiveLock(_) | Modifier::AbilityLock(_)))
}

/// Where the lock's source must be for it to hold, and its owner when it is.
fn lock_position(g: &Game, c: CardId, p: &Passive) -> Option<usize> {
    match &p.modifier {
        Modifier::ActiveLock(_) => (0..2usize).find(|q| g.st.active_pokemon(*q) == Some(c)),
        Modifier::AbilityLock(l) => locker_owner(g, c, l.locker),
        _ => None,
    }
}

/// The player whose card `me` is where a lock of kind `locker` needs it (a Pokémon in a Bench spot, in play,
/// the Stadium in play).
fn locker_owner(g: &Game, me: CardId, locker: Locker) -> Option<usize> {
    match locker {
        Locker::BenchOfEitherSide => {
            let mut found = None;
            for q in 0..2usize {
                let pl = &g.st.players[q];
                if pl.bench.iter().any(|b| g.st.slot_pokemon(q, *b) == Some(me)) {
                    found = Some(q);
                }
            }
            found
        }
        Locker::InPlayEitherSide => {
            let in_play = (0..2usize).any(|q| g.st.players[q].in_play().iter().any(|s| g.st.slot_pokemon(q, *s) == Some(me)));
            if in_play {
                g.st.locate(me).and_then(|x| x.owner())
            } else {
                None
            }
        }
        Locker::StadiumInPlay => (g.st.stadium_card() == Some(me)).then(|| g.st.locate(me).and_then(|x| x.owner()).unwrap_or_else(|| g.st.owner(me))),
    }
}

/// The Ability of a Pokémon card that a lock would turn off (its first Ability).
fn first_ability(g: &Game, c: CardId) -> Option<crate::effects::PowerRef> {
    g.st.cdef(c).powers.iter().position(|pw| pw.power_type == PowerType::Ability as u8).map(|i| crate::effects::PowerRef { card: c, index: i as u8 })
}

/// Would lock `pa` of `a`, in place, turn off the Ability of lock source `s` where it is now? Position and
/// coverage only: the lock's own probes aren't run.
fn lock_covers(g: &mut Game, a: CardId, pa: &Passive, s: CardId) -> R<bool> {
    let Some(owner) = lock_position(g, a, pa) else { return Ok(false) };
    let Some(power) = first_ability(g, s) else { return Ok(false) };
    let slot = match g.st.locate(s) {
        Some(ListRef::Slot(q, sl)) => SlotRef::new(q as usize, sl),
        _ => return Ok(false),
    };
    match &pa.modifier {
        Modifier::ActiveLock(l) => {
            if !active_lock_subject(g, *l, power, false) {
                return Ok(false);
            }
            Ok(match l {
                ActiveLock::MidnightFluttering => {
                    // Hide 'n' Sneak takes precedence over Midnight Fluttering.
                    slot == SlotRef::new(1 - owner, g.st.players[1 - owner].active) && !g.st.cdef(s).powers.iter().any(|pw| pw.name == "Hide 'n' Sneak")
                }
                ActiveLock::Initialization => {
                    let d = g.st.cdef(s);
                    !d.has_tag(tag::FUTURE) && d.has_rule_box()
                }
            })
        }
        Modifier::AbilityLock(l) => {
            if !power_subject(g, &l.powers, power) || !pred(g, s, &l.card) {
                return Ok(false);
            }
            Ok(slot_pred_m(g, a, slot, &l.slot).unwrap_or(false))
        }
        _ => Ok(false),
    }
}

/// Did lock source `a` take hold before `s`? Stamps decide; with neither stamped (they take hold together) the
/// turn player's card is first.
fn lock_earlier(g: &Game, a: CardId, s: CardId) -> bool {
    let (sa, ss) = (g.st.cards[a as usize].lock_stamp, g.st.cards[s as usize].lock_stamp);
    match (sa, ss) {
        (0, 0) => lock_turn_order(g, a) <= lock_turn_order(g, s),
        (0, _) => false,
        (_, 0) => true,
        _ => sa < ss,
    }
}

/// Sort key of simultaneous take-holds: the turn player's cards first, then by card.
fn lock_turn_order(g: &Game, c: CardId) -> (bool, CardId) {
    let owner = g.st.locate(c).and_then(|l| l.owner()).unwrap_or_else(|| g.st.owner(c));
    (owner != g.st.active_player as usize, c)
}

/// Lock `a` covers lock source `s`: does it win? It does when `s` can't turn `a` off in return, or `a` took
/// hold first.
fn lock_beats(g: &mut Game, a: CardId, s: CardId) -> R<bool> {
    let mut mutual = false;
    for ps in lock_passives(g, s) {
        if lock_covers(g, s, ps, a)? {
            mutual = true;
            break;
        }
    }
    Ok(!mutual || lock_earlier(g, a, s))
}

/// Does the source of lock `p` have its Ability (it is in place, `owner`'s)? Probed as the lock itself does.
/// A Stadium has none to turn off.
fn lock_has_ability(g: &mut Game, c: CardId, p: &Passive, owner: usize) -> bool {
    if p.origin != RuleSource::Ability {
        return true;
    }
    let real = |g: &mut Game, index: u8| g.run_fx(Effect::Power { p: owner as u8, power: crate::effects::PowerRef { card: c, index }, card: c, target: None, probe: false }).is_ok();
    match &p.modifier {
        Modifier::ActiveLock(_) => real(g, 0),
        Modifier::AbilityLock(l) => match l.probe {
            LockerProbe::Generic => !is_ability_blocked(g, owner, c, None),
            LockerProbe::OwnPower(i) => real(g, i),
            LockerProbe::StadiumOnSlot => true,
        },
        _ => true,
    }
}

/// Can whether this Ability lock's source has its Ability (and so whether the lock holds) depend on what is
/// attached to a Pokémon? Its source is a Pokémon in play or the Stadium (never an attached card), so an
/// attachment changes nothing about where it is; what can change is the answer of the locks that cover it,
/// read through the same declarations: the spot predicate ([`SlotPred::reads_attached`]: the type, the Energy,
/// a Tool ...) and the Stadium-effect probe (effects on the spot can block the Stadium). The card predicate
/// reads the printed card. The two `ActiveLock`s read the Active Spot, the printed card, its Rule Box and
/// tags: never this.
pub const fn lock_reads_attached(l: &AbilityLockSpec) -> bool {
    l.slot.reads_attached() || matches!(l.probe, LockerProbe::StadiumOnSlot)
}

/// Does the card declare an Ability lock (`ActiveLock` / `AbilityLock`)?
pub const fn declares_ability_lock(passives: &[Passive]) -> bool {
    let mut i = 0;
    while i < passives.len() {
        if matches!(passives[i].modifier, Modifier::ActiveLock(_) | Modifier::AbilityLock(_)) {
            return true;
        }
        i += 1;
    }
    false
}

/// `lock_sync` after an Attach, a MoveEnergy or a MoveTool. Where every Pokémon and the Stadium are didn't
/// change, so the stamps can only change when a lock source's Ability can be turned on or off by an attached
/// card: some card of the game declares such a lock (`k::DECLARES_ATTACHED_LOCK`). Otherwise the sync is
/// skipped; `PTCG_VERIFY_CACHE=1` runs it anyway and asserts that it changed nothing.
pub(crate) fn lock_sync_attached(g: &mut Game) {
    if g.kinds_present.has(crate::effects::k::DECLARES_ATTACHED_LOCK) {
        lock_sync(g);
        return;
    }
    if !crate::game::verify_cache() || g.lock_syncing || !g.kinds_present.has(crate::effects::k::CHECK_POKEMON_POWERS) {
        return;
    }
    let before: Vec<u32> = (0..g.st.n_cards).map(|c| g.st.cards[c as usize].lock_stamp as u32).collect();
    let counter = g.st.ability_lock_order_counter;
    lock_sync(g);
    let after: Vec<u32> = (0..g.st.n_cards).map(|c| g.st.cards[c as usize].lock_stamp as u32).collect();
    assert!(
        before == after && counter == g.st.ability_lock_order_counter,
        "lock_sync after an attaching event changed the take-hold stamps in a game with no DECLARES_ATTACHED_LOCK"
    );
}

/// Keep the take-hold stamps up to date after the board changed: a lock source that holds (in place, its
/// Ability on) and has no stamp gets the next one, the turn player's first; one that doesn't hold loses its
/// stamp. Releasing a lock that was turned off can let the other take hold, so it repeats until nothing
/// changes.
pub(crate) fn lock_sync(g: &mut Game) {
    if g.lock_syncing || !g.kinds_present.has(crate::effects::k::CHECK_POKEMON_POWERS) {
        return;
    }
    // Only a card that declares an Ability lock can hold one or carry a stamp (`Game::lock_cards`).
    let lockers = g.lock_cards;
    let is_locker = |c: CardId| lockers[(c >> 6) as usize] & (1u64 << (c & 63)) != 0;
    if crate::game::verify_cache() {
        for c in 0..g.st.n_cards {
            assert_eq!(lock_passives(g, c).next().is_some(), is_locker(c), "Game::lock_cards is stale for card {c}");
        }
    }
    g.lock_syncing = true;
    for _ in 0..4 {
        // The lock sources in play (the Pokémon, Active first, then the Stadium), then the stamped cards that
        // left play, by card.
        let mut sources: SVec<CardId, 24> = SVec::new();
        for q in 0..2usize {
            for s in g.st.players[q].in_play().iter() {
                if let Some(c) = g.st.slot_pokemon(q, *s) {
                    if is_locker(c) {
                        sources.push(c);
                    }
                }
            }
        }
        if let Some(c) = g.st.stadium_card() {
            if is_locker(c) {
                sources.push(c);
            }
        }
        for (w, bits) in lockers.iter().enumerate() {
            let mut b = *bits;
            while b != 0 {
                let c = (w * 64 + b.trailing_zeros() as usize) as CardId;
                b &= b - 1;
                if g.st.cards[c as usize].lock_stamp != 0 && !sources.contains(&c) {
                    sources.push(c);
                }
            }
        }
        if sources.is_empty() {
            break;
        }
        let mut changed = false;
        let mut taking: Vec<CardId> = Vec::new();
        for c in sources.iter().copied() {
            let mut holds = false;
            for p in lock_passives(g, c) {
                if let Some(owner) = lock_position(g, c, p) {
                    holds |= lock_has_ability(g, c, p, owner);
                }
            }
            let stamp = g.st.cards[c as usize].lock_stamp;
            if !holds && stamp != 0 {
                g.st.cards[c as usize].lock_stamp = 0;
                changed = true;
            } else if holds && stamp == 0 {
                taking.push(c);
            }
        }
        taking.sort_by_key(|c| lock_turn_order(g, *c));
        for c in taking {
            g.st.ability_lock_order_counter = g.st.ability_lock_order_counter.saturating_add(1);
            g.st.cards[c as usize].lock_stamp = g.st.ability_lock_order_counter;
            changed = true;
        }
        if !changed {
            break;
        }
    }
    g.lock_syncing = false;
}

/// `IS_POWER_SUBJECT_TO_ABILITY_LOCK` with the lock's options.
fn active_lock_subject(g: &Game, l: ActiveLock, power: crate::effects::PowerRef, probe: bool) -> bool {
    if power.index == PROBE_GENERIC {
        return true;
    }
    let d = &g.st.cdef(power.card).powers[power.index as usize];
    if d.power_type != PowerType::Ability as u8 || d.exempt_from_ability_lock {
        return false;
    }
    match l {
        ActiveLock::MidnightFluttering => probe || d.name != "Midnight Fluttering",
        ActiveLock::Initialization => !d.exempt_from_initialize && !d.use_from_hand && !d.use_from_discard,
    }
}

/// The `HANDLE_ABILITY_LOCK` callback: is `card`'s Ability locked by `me`? `player` is the player of the
/// effect being checked, `power_effect` whether it is a use of an Ability.
fn active_lock_applies(g: &mut Game, me: CardId, l: ActiveLock, player: usize, card: CardId, power_effect: bool) -> R<bool> {
    let own = crate::effects::PowerRef { card: me, index: 0 };
    match l {
        ActiveLock::MidnightFluttering => {
            // Only a Flutter Mane in its owner's Active Spot locks, and only the opponent's Active Pokémon; a card
            // anywhere else (a deck, a list of cards being looked at) is neither.
            let Some(owner) = g.st.locate(me).and_then(|l| l.owner()) else { return Ok(false) };
            if g.st.active_pokemon(owner) != Some(me) {
                return Ok(false);
            }
            let opponent = 1 - owner;
            if g.st.locate(card) != Some(ListRef::Slot(opponent as u8, g.st.players[opponent].active)) {
                return Ok(false);
            }
            // Hide 'n' Sneak takes precedence over Midnight Fluttering.
            if g.st.cdef(card).powers.iter().any(|pw| pw.name == "Hide 'n' Sneak") {
                return Ok(false);
            }
            // LOCKER_ABILITY_APPLIES: a lock source is turned off only by a lock that wins over it.
            if lock_passives(g, card).next().is_some() && !lock_beats(g, me, card)? {
                return Ok(false);
            }
            Ok(g.run_fx(Effect::Power { p: owner as u8, power: own, card: me, target: None, probe: false }).is_ok())
        }
        ActiveLock::Initialization => {
            if g.st.active_pokemon(player) != Some(me) && g.st.active_pokemon(1 - player) != Some(me) {
                return Ok(false);
            }
            // A card in the hand is judged by its printed data (docs/rulings/RULES.md): only Pokémon in play are locked.
            let slot = match g.st.locate(card) {
                Some(ListRef::Slot(q, s)) => SlotRef::new(q as usize, s),
                _ => return Ok(false),
            };
            let d = g.st.cdef(card);
            if d.has_tag(tag::FUTURE) || !d.has_rule_box() {
                return Ok(false);
            }
            let locker_owner = if g.st.active_pokemon(player) == Some(me) { player } else { 1 - player };
            // LOCKER_ABILITY_APPLIES
            if lock_passives(g, card).next().is_some() && !lock_beats(g, me, card)? {
                return Ok(false);
            }
            if g.run_fx(Effect::Power { p: locker_owner as u8, power: own, card: me, target: None, probe: false }).is_err() {
                return Ok(false);
            }
            if power_effect {
                // CAN_APPLY_LOCK_TO_TARGET
                return Ok(match g.run_fx(Effect::EffectOfAbility { p: locker_owner as u8, power: own, card: me, target: Some(slot), cause: crate::cause::Cause::new(crate::cause::CauseKind::Ability, Some(me), locker_owner as u8) }) {
                    Ok((Effect::EffectOfAbility { target, .. }, _)) => target.is_some(),
                    _ => false,
                });
            }
            Ok(true)
        }
    }
}

fn active_lock(g: &mut Game, me: CardId, e: EffId, l: ActiveLock) -> R {
    // Initialization can't change the effect of its own Ability on a Future Pokémon.
    if l == ActiveLock::Initialization {
        if let Effect::EffectOfAbility { power, card, target: Some(t), .. } = *g.e(e) {
            if card == me && power == (crate::effects::PowerRef { card: me, index: 0 }) {
                if let Some(c) = g.st.slot_pokemon(t.p as usize, t.s) {
                    if g.st.cdef(c).has_tag(tag::FUTURE) {
                        if let Effect::EffectOfAbility { target, .. } = g.e_mut(e) {
                            *target = None;
                        }
                    }
                }
            }
        }
    }
    match *g.e(e) {
        Effect::CheckPokemonPowers { p, target, powers } => {
            if active_lock_applies(g, me, l, p as usize, target, false)? {
                let mut out = SVec::new();
                for pw in powers.iter() {
                    if !active_lock_subject(g, l, *pw, false) {
                        out.push(*pw);
                    }
                }
                if let Effect::CheckPokemonPowers { powers, .. } = g.e_mut(e) {
                    *powers = out;
                }
            }
        }
        Effect::Power { p, power, card, probe, .. } => {
            if active_lock_subject(g, l, power, probe) && active_lock_applies(g, me, l, p as usize, card, true)? {
                crate::bail!("BLOCKED_BY_ABILITY");
            }
        }
        _ => {}
    }
    Ok(())
}

/// A guard evaluated with checked reads (Energy provided, types as the game checks them).
fn guard_ok_m(g: &mut Game, me: CardId, origin: RuleSource, owner: usize, guard: &Cond) -> R<bool> {
    let f = run::Frame::passive(g, me, owner, origin);
    cond_m(g, me, &f, guard)
}

/// A Benched Pokémon whose counters come from the opponent's Pokémon (Battle Cage).

fn bench_size(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &BenchSizeSpec) -> R {
    let Effect::CheckTableState { bench_sizes } = *g.e(e) else { return Ok(()) };
    if locate(g, me, origin).is_none() {
        return Ok(());
    }
    let mut sizes = bench_sizes;
    for (p, size) in sizes.iter_mut().enumerate() {
        if guard_ok(g, me, origin, p, &d.guard) {
            *size = d.size;
        }
    }
    if let Effect::CheckTableState { bench_sizes } = g.e_mut(e) {
        *bench_sizes = sizes;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Heavy Baton

// S3-4 appends: attack blocks, play blocks, survive on 10 HP, Shred

/// Which effect an attack block reacts to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttackBlockOn {
    /// Any attack effect while this card is its player's Active Pokémon.
    ActiveAttack,
    /// Using an attack of the Pokémon this card is part of.
    UseAttack,
}

/// "This Pokémon can't attack unless ...": the attack is refused with `error` unless `unless` holds
/// (read for the attacking player), while the Ability works.
pub struct BlockAttackSpec {
    pub on: AttackBlockOn,
    pub unless: Cond,
    pub error: &'static str,
}

fn block_attack(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, b: &BlockAttackSpec) -> R {
    let (p, source) = match (b.on, *g.e(e)) {
        (AttackBlockOn::ActiveAttack, Effect::Attack { p, .. }) => (p as usize, None),
        (AttackBlockOn::UseAttack, Effect::UseAttack { p, source, .. }) => (p as usize, Some(source)),
        _ => return Ok(()),
    };
    match attack_block_error(g, me, origin, b, p, source) {
        Some(code) => crate::bail!(code),
        None => Ok(()),
    }
}

/// The error code when the block `b` of `me` stops player `p`'s attack (`source`: the attacking slot of a
/// `UseAttack`; `None` for the `Attack` effect, which only the Active Pokémon's own block reads).
fn attack_block_error(g: &mut Game, me: CardId, origin: RuleSource, b: &BlockAttackSpec, p: usize, source: Option<SlotRef>) -> Option<&'static str> {
    match source {
        None => {
            if g.st.active_pokemon(p) != Some(me) {
                return None;
            }
        }
        Some(src) => {
            if !g.st.slot(src.p as usize, src.s).cards.contains(me) {
                return None;
            }
        }
    }
    let at = locate(g, me, origin)?;
    if blocked(g, me, origin, at, None) || guard_ok(g, me, origin, p, &b.unless) {
        return None;
    }
    Some(b.error)
}

/// Legality: does a block declared by `me` (`BlockAttack`) stop player `p` from attacking? Both blocks are
/// read: the one on the `UseAttack` (the Pokémon's own attacks) and the one on the `Attack` effect (while it
/// is the Active Pokémon).
pub fn attack_blocked_by(g: &mut Game, me: CardId, p: usize) -> bool {
    let passives: &'static [Passive] = crate::cards::spec_for(g.st.cards[me as usize].def).map_or(&[], |s| s.passives);
    let active = SlotRef::new(p, g.st.players[p].active);
    for ps in passives {
        if let Modifier::BlockAttack(b) = &ps.modifier {
            let source = match b.on {
                AttackBlockOn::ActiveAttack => None,
                AttackBlockOn::UseAttack => Some(active),
            };
            if attack_block_error(g, me, ps.origin, b, p, source).is_some() {
                return true;
            }
        }
    }
    false
}

/// "If the Pokémon has full HP and would be Knocked Out by damage from an opponent's attack, it is not Knocked Out
/// and its remaining HP becomes 10 instead; then discard this card."
fn survive_on_ten_tool(g: &mut Game, me: CardId, e: EffId, origin: RuleSource) -> R {
    let (t, damage, attacker) = match *g.e(e) {
        Effect::PutDamage { b, damage, .. } => (b.target, damage, b.player as usize),
        _ => return Ok(()),
    };
    if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
        return Ok(());
    }
    let owner = t.p as usize;
    // Only damage from an attack by the opponent's Pokémon.
    if attacker == owner || !is_attack_phase(g) {
        return Ok(());
    }
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if blocked(g, me, origin, at, Some(t)) || g.st.slot(owner, t.s).damage != 0 {
        return Ok(());
    }
    let hp = crate::derived::hp(g, owner, t.s)?;
    if damage < hp {
        return Ok(());
    }
    if let Effect::PutDamage { survive_on_ten_hp, .. } = g.e_mut(e) {
        *survive_on_ten_hp = true;
    }
    for (s, _, _) in for_each_pokemon(g, owner, PlayerType::BottomPlayer).iter().copied() {
        if g.st.slot(owner, s).tools.contains(me) {
            // "Then, discard this card": a LeavePlay of the attached Tool by its own text.
            let cause = crate::cause::Cause::of_origin(origin, me, owner as u8);
            crate::engine::knockout::leave_play_cards(g, SlotRef::new(owner, s), &[me], super::event::RulesZone::Discard, cause, None)?;
        }
    }
    Ok(())
}

/// Shred: the damage of the attacks of Pokémon matching `attacker` isn't affected by effects on the Defending Pokémon.
fn shred_flags(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, attacker: &SlotPred) -> R {
    let (source, attack) = match *g.e(e) {
        Effect::Attack { source, attack, .. } => (source, attack),
        _ => return Ok(()),
    };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if !slot_pred_m(g, me, source, attacker)? || blocked(g, me, origin, at, None) {
        return Ok(());
    }
    // `effect.attack.shredAttack = true` (a per-card attack object write; a copy-attack clone carries its own flag).
    if !attack.is_clone() && !g.st.cdef(attack.card).attacks[attack.idx()].shred_attack {
        g.st.cards[attack.card as usize].attack_shred |= 1u8 << attack.idx();
    }
    if let Effect::Attack { ignore_defender_effects, .. } = g.e_mut(e) {
        *ignore_defender_effects = true;
    }
    Ok(())
}

#[cfg(test)]
mod lock_tests {
    //! Take-hold stamps and precedence between locks (docs/rulings/RULES.md). No two locks in the pool turn each other
    //! off, so the mutual case is tested on the stamps directly.
    use super::*;
    use serde_json::json;

    const FM: &str = "Flutter Mane PRE 43";
    const IT: &str = "Iron Thorns ex PRE 32";
    const DURA: &str = "Duraludon PRE 69";

    fn game(sc: serde_json::Value) -> Game {
        let deck: Vec<u16> = (0..4)
            .map(|_| FM)
            .chain((0..4).map(|_| IT))
            .chain((0..4).map(|_| DURA))
            .chain((0..48).map(|_| "Metal Energy MEE 8"))
            .map(|n| crate::carddb::def_by_full_name(n).unwrap())
            .collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    fn top(g: &Game, p: usize, bench: Option<usize>) -> CardId {
        let s = match bench {
            None => g.st.players[p].active,
            Some(i) => g.st.players[p].bench.as_slice()[i],
        };
        g.st.slot_pokemon(p, s).unwrap()
    }

    fn stamp(g: &Game, c: CardId) -> u16 {
        g.st.cards[c as usize].lock_stamp
    }

    #[test]
    fn one_way_lock_wins_whatever_the_order() {
        // Flutter Mane (mine, Active) turns off Iron Thorns ex's Initialization; Initialization can't
        // touch Flutter Mane (no Rule Box).
        let g = game(json!({"me": {"reset": true, "active": FM}, "opp": {"reset": true, "active": IT}}));
        let me = g.st.active_player as usize;
        assert!(stamp(&g, top(&g, me, None)) > 0, "Flutter Mane holds");
        assert_eq!(stamp(&g, top(&g, 1 - me, None)), 0, "Iron Thorns ex's lock is off");
        // The same with the sides swapped: the turn player has Iron Thorns ex.
        let g = game(json!({"me": {"reset": true, "active": IT}, "opp": {"reset": true, "active": FM}}));
        let me = g.st.active_player as usize;
        assert_eq!(stamp(&g, top(&g, me, None)), 0);
        assert!(stamp(&g, top(&g, 1 - me, None)) > 0);
    }

    #[test]
    fn simultaneous_take_hold_is_stamped_turn_player_first_and_stays() {
        let mut g = game(json!({"me": {"reset": true, "active": FM}, "opp": {"reset": true, "active": FM}}));
        let me = g.st.active_player as usize;
        let (a, b) = (top(&g, me, None), top(&g, 1 - me, None));
        assert_eq!((stamp(&g, a), stamp(&g, b)), (1, 2), "the turn player's lock took hold first");
        // The turn passes: the order doesn't swap.
        g.st.active_player = (1 - me) as u8;
        lock_sync(&mut g);
        assert_eq!((stamp(&g, a), stamp(&g, b)), (1, 2));
        assert!(lock_earlier(&g, a, b) && !lock_earlier(&g, b, a));
    }

    /// The player's own switch (a Trainer's), the ChangeActive event.
    fn switch(g: &mut Game, me: usize, slot: crate::state::SlotId) {
        let c = crate::engine::change_active::ChangeActiveView::of(g, me, Some(slot), crate::spec::event::ActiveChange::Switch, crate::cause::Cause::new(crate::cause::CauseKind::Trainer, None, me as u8));
        assert!(crate::engine::change_active::change_active(g, c).unwrap());
    }

    #[test]
    fn stamps_follow_take_hold_and_release() {
        // Iron Thorns ex (opp) is Active; Flutter Mane waits on my Bench.
        let mut g = game(json!({"me": {"reset": true, "active": DURA, "bench": [{"card": FM}]}, "opp": {"reset": true, "active": IT}}));
        let me = g.st.active_player as usize;
        let (fm, it) = (top(&g, me, Some(0)), top(&g, 1 - me, None));
        assert_eq!((stamp(&g, fm), stamp(&g, it)), (0, 1), "Iron Thorns ex holds, Flutter Mane isn't Active");
        // Flutter Mane comes in: it turns Iron Thorns ex's lock off though that took hold first.
        let slot = g.st.players[me].bench.as_slice()[0];
        switch(&mut g, me, slot);
        assert_eq!((stamp(&g, fm), stamp(&g, it)), (2, 0));
        // Flutter Mane leaves: Iron Thorns ex is released and takes hold now.
        let slot = g.st.players[me].bench.as_slice()[0];
        switch(&mut g, me, slot);
        assert_eq!((stamp(&g, fm), stamp(&g, it)), (0, 3));
    }

    #[test]
    fn mutual_locks_the_earlier_stamp_wins() {
        let mut g = game(json!({"me": {"reset": true, "active": FM}, "opp": {"reset": true, "active": FM}}));
        let me = g.st.active_player as usize;
        let (a, b) = (top(&g, me, None), top(&g, 1 - me, None));
        // Pretend the opponent's lock took hold first, then ours.
        g.st.cards[a as usize].lock_stamp = 5;
        g.st.cards[b as usize].lock_stamp = 3;
        assert!(lock_earlier(&g, b, a) && !lock_earlier(&g, a, b));
        // A lock that holds beats one that doesn't yet; with neither, the turn player's card is first.
        g.st.cards[a as usize].lock_stamp = 0;
        assert!(lock_earlier(&g, b, a) && !lock_earlier(&g, a, b));
        g.st.cards[b as usize].lock_stamp = 0;
        assert!(lock_earlier(&g, a, b) && !lock_earlier(&g, b, a));
    }
}

#[cfg(test)]
mod ace_spec_tests {
    //! Genesect's ACE Nullifier blocks ACE SPEC cards played from the hand only (A-PC6), whatever attaches them
    //! (id25, id230). No pool card attaches an ACE SPEC Energy by an effect (every pool effect that attaches
    //! from the hand, deck or discard pile takes Basic Energy), so this can't be a scenario.
    use super::*;
    use serde_json::json;

    fn attach(from: &str) -> Result<(), &'static str> {
        let mut names: Vec<&str> = vec!["Genesect SFA 40"; 4];
        names.extend(["Enriching Energy SSP 191"; 1]);
        names.extend(["Sacred Charm PFL 93"; 4]);
        names.extend(["Duraludon PRE 69"; 4]);
        names.extend(["Metal Energy MEE 8"; 47]);
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        let sc = json!({
            "me": {"reset": true, "active": "Duraludon PRE 69", from: ["Enriching Energy SSP 191"]},
            "opp": {"reset": true, "active": "Genesect SFA 40", "active_tool": "Sacred Charm PFL 93"}
        });
        crate::scenario::apply(&mut g, &sc).unwrap();
        let me = g.st.active_player as usize;
        let card = g.st.cards.iter().position(|c| c.def == crate::carddb::def_by_full_name("Enriching Energy SSP 191").unwrap() && c.owner as usize == me).unwrap() as CardId;
        let target = SlotRef::new(me, g.st.players[me].active);
        // An Ability attaching it (id25, id230: any cause; the lock reads the event's source zone).
        let cause = crate::cause::Cause::new(crate::cause::CauseKind::Ability, None, me as u8);
        let (_, source) = crate::engine::enter::source_of(&g, card).unwrap();
        let v = crate::engine::attach::attach_view(&g, card, target, source, false, cause);
        crate::engine::attach::check_attach(&mut g, &v).map_err(|e| e.0)
    }

    /// Enriching Energy draws 4 cards only when attached from the hand (hand size after minus before).
    fn draws(from: &str) -> i32 {
        let mut names: Vec<&str> = vec!["Enriching Energy SSP 191"; 1];
        names.extend(["Duraludon PRE 69"; 4]);
        names.extend(["Metal Energy MEE 8"; 55]);
        let deck: Vec<u16> = names.iter().map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        let sc = json!({"me": {"reset": true, "active": "Duraludon PRE 69", from: ["Enriching Energy SSP 191"]}, "opp": {"reset": true, "active": "Duraludon PRE 69"}});
        crate::scenario::apply(&mut g, &sc).unwrap();
        let me = g.st.active_player as usize;
        let card = g.st.cards.iter().position(|c| c.def == crate::carddb::def_by_full_name("Enriching Energy SSP 191").unwrap() && c.owner as usize == me).unwrap() as CardId;
        let before = g.st.players[me].hand.len() as i32;
        let target = SlotRef::new(me, g.st.players[me].active);
        // An effect attaching it (APR C-09: "when you attach this card from your hand" counts it too).
        let cause = crate::cause::Cause::new(crate::cause::CauseKind::Ability, None, me as u8);
        assert!(crate::engine::attach::attach(&mut g, card, target, cause).unwrap());
        g.settle().ok();
        g.st.players[me].hand.len() as i32 - before
    }

    #[test]
    fn enriching_energy_draws_only_when_attached_from_the_hand() {
        assert_eq!(draws("hand"), 3, "the card leaves the hand, 4 cards are drawn");
        assert_eq!(draws("discard"), 0);
    }

    #[test]
    fn only_a_card_played_from_the_hand_is_blocked() {
        assert_eq!(attach("hand"), Err("BLOCKED_BY_EFFECT"));
        assert_eq!(attach("discard"), Ok(()));
    }
}

#[cfg(test)]
mod play_lock_tests {
    //! `event_locked` and `ability_off` on the lock cards (the query execution and legality share).
    use super::*;
    use serde_json::json;

    /// The lock answer for player `p` playing the Trainer `card` from the hand (the PlayTrainer event).
    fn play_locked(g: &mut Game, p: usize, card: CardId) -> Option<&'static str> {
        let cause = crate::cause::Cause::rule(crate::cause::RuleWhich::Action, p as u8);
        let v = crate::engine::play_trainer::play_view(g, card, crate::spec::event::TrainerUse::Played, crate::spec::event::RulesZone::Hand, cause);
        event_locked(g, &v).unwrap()
    }

    const NAMES: [&str; 20] = [
        "Boss's Orders ASC 183",
        "Frillish WHT 44",
        "Jellicent ex WHT 45",
        "Iron Thorns ex PRE 32",
        "Flutter Mane PRE 43",
        "Team Rocket's Ekans DRI 112",
        "Team Rocket's Arbok DRI 113",
        "Hoothoot PRE 77",
        "Noctowl PRE 78",
        "Team Rocket's Watchtower ASC 210",
        "Meowth ex POR 62",
        "Potion POR 83",
        "Sacred Charm PFL 93",
        "Enriching Energy SSP 191",
        "Genesect SFA 40",
        "Duraludon PRE 69",
        "Palafin ex PRE 151",
        "Finizen TWM 59",
        "Palafin TWM 60",
        "Antique Root Fossil SCR 130",
    ];

    fn game(sc: serde_json::Value) -> Game {
        let mut deck: Vec<u16> = NAMES.iter().flat_map(|n| (0..if n.starts_with("Enriching") { 1 } else { 3 }).map(move |_| *n)).map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        while deck.len() < 60 {
            deck.push(crate::carddb::def_by_full_name("Water Energy MEE 3").unwrap());
        }
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g.settle().ok();
        crate::scenario::apply(&mut g, &sc).unwrap();
        g
    }

    fn hand(g: &Game, p: usize, name: &str) -> CardId {
        let def = crate::carddb::def_by_full_name(name).unwrap();
        g.st.players[p].hand.iter().find(|c| g.st.cards[*c as usize].def == def).unwrap_or_else(|| panic!("no {name} in the hand"))
    }

    fn active(g: &Game, p: usize) -> CardId {
        g.st.active_pokemon(p).unwrap()
    }

    fn bench(g: &Game, p: usize, i: usize) -> CardId {
        g.st.slot_pokemon(p, g.st.players[p].bench.as_slice()[i]).unwrap()
    }

    /// The lock answer for the Evolve (`evolve`) or EnterPlay event of `card` from the hand, onto the active spot
    /// of `owner` (the event's owner), by the rule.
    fn event_lock(g: &mut Game, owner: usize, card: CardId, evolve: bool) -> Option<&'static str> {
        use crate::engine::enter::{enter_view, evolve_view};
        let cause = crate::cause::Cause::rule(crate::cause::RuleWhich::Action, owner as u8);
        let t = crate::effects::SlotRef::new(owner, g.st.players[owner].active);
        let v = if evolve {
            evolve_view(g, Some(card), t, crate::spec::event::RulesZone::Hand, crate::spec::event::EvolvePath::Rule, cause).unwrap()
        } else {
            enter_view(g, card, t, crate::spec::event::RulesZone::Hand, crate::spec::event::EnterMode::Rule, cause)
        };
        event_locked(g, &v).unwrap()
    }

    /// The lock answer for the Attach event of `card` from where it is onto `owner`'s Active Pokémon, caused by
    /// `cause` (the one lock query, `derived::event_locked`, as the Attach routine asks it).
    fn attach_lock(g: &mut Game, owner: usize, card: CardId, cause: crate::cause::Cause) -> Option<&'static str> {
        let t = crate::effects::SlotRef::new(owner, g.st.players[owner].active);
        let (_, source) = crate::engine::enter::source_of(g, card).unwrap();
        let v = crate::engine::attach::attach_view(g, card, t, source, false, cause);
        crate::derived::event_locked(g, &v).unwrap()
    }

    fn rule(p: usize) -> crate::cause::Cause {
        crate::cause::Cause::rule(crate::cause::RuleWhich::Action, p as u8)
    }

    fn ability(p: usize) -> crate::cause::Cause {
        crate::cause::Cause::new(crate::cause::CauseKind::Ability, None, p as u8)
    }

    /// A copy of `name` of `p`'s in the deck (or the Prizes after a reset): not in the hand.
    fn in_deck(g: &Game, p: usize, name: &str) -> CardId {
        let def = crate::carddb::def_by_full_name(name).unwrap();
        let pl = &g.st.players[p];
        pl.deck.iter().chain(pl.prizes.iter().flat_map(|z| z.iter())).find(|c| g.st.cards[*c as usize].def == def).unwrap()
    }

    /// The lock each attack leaves, applied to player `p` as the attack does, with what it stops.
    #[test]
    fn attack_locks_stop_what_their_text_says_and_expire() {
        use crate::engine::phase::{apply_play_lock, tick_play_locks_at_end_of_turn};
        // (the card's declaration, the card it stops, the cards it doesn't)
        static ITEM: LockDecl = LockDecl::on(PLAY_ITEM_FROM_HAND, "BLOCKED_BY_EFFECT");
        static SUPPORTER: LockDecl = LockDecl::on(PLAY_SUPPORTER_FROM_HAND, "BLOCKED_BY_EFFECT");
        static STADIUM: LockDecl = LockDecl::on(PLAY_STADIUM_FROM_HAND, "BLOCKED_BY_EFFECT");
        let mut g = game(json!({"me": {"reset": true, "active": "Hoothoot PRE 77", "hand": ["Potion POR 83", "Noctowl PRE 78", "Team Rocket's Watchtower ASC 210", "Boss's Orders ASC 183"]},
            "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let me = g.st.active_player as usize;
        let potion = hand(&g, me, "Potion POR 83");
        let tower = hand(&g, me, "Team Rocket's Watchtower ASC 210");
        let research = hand(&g, me, "Boss's Orders ASC 183");
        let cases: [(&'static LockDecl, CardId); 3] = [
            (&ITEM, potion),         // Budew, Frillish, Galvantula ex
            (&SUPPORTER, research), // Scream Tail ex
            (&STADIUM, tower),       // Chi-Yu
        ];
        for (decl, stops) in cases {
            apply_play_lock(&mut g.st.players[me], decl, 1, 0);
            for c in [potion, research, tower] {
                let want = (c == stops).then_some("BLOCKED_BY_EFFECT");
                assert_eq!(play_locked(&mut g, me, c), want, "{} under a lock on {}", g.st.cdef(c).name, g.st.cdef(stops).name);
            }
            // The opponent of the locked player isn't stopped.
            assert_eq!(play_locked(&mut g, 1 - me, stops), None);
            tick_play_locks_at_end_of_turn(&mut g.st.players[1 - me]);
            assert!(play_locked(&mut g, me, stops).is_some(), "the other player's turn doesn't end it");
            tick_play_locks_at_end_of_turn(&mut g.st.players[me]);
            assert_eq!(play_locked(&mut g, me, stops), None, "gone at the end of the locked player's turn");
        }
        // A used Supporter isn't played (id2226: "can't play Supporter cards" doesn't stop Look-Alike Show).
        apply_play_lock(&mut g.st.players[me], &SUPPORTER, 1, 0);
        let attack = crate::cause::Cause::attack(me as u8, None, crate::state::AttackRef { card: 0, index: 0 });
        let used = crate::engine::play_trainer::play_view(&g, research, crate::spec::event::TrainerUse::Used, crate::spec::event::RulesZone::Hand, attack);
        assert_eq!(event_locked(&mut g, &used).unwrap(), None);
        // Seismitoad's coin-gated lock isn't a legality check (heads lets the play through): the lock query doesn't answer
        // it; the play flips (`coin_gate`).
        static QUAKING: LockDecl = LockDecl { error: "BLOCKED_BY_EFFECT", forbids: PLAY_TRAINER_FROM_HAND, coin: CoinGate::TailsDiscardsCard };
        let mut g2 = game(json!({"me": {"reset": true, "active": "Hoothoot PRE 77", "hand": ["Potion POR 83"]}, "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let potion = hand(&g2, me, "Potion POR 83");
        apply_play_lock(&mut g2.st.players[me], &QUAKING, 1, 0);
        assert_eq!(play_locked(&mut g2, me, potion), None);
        let v = crate::engine::play_trainer::play_view(&g2, potion, crate::spec::event::TrainerUse::Played, crate::spec::event::RulesZone::Hand, crate::cause::Cause::rule(crate::cause::RuleWhich::Action, me as u8));
        assert!(coin_gate(&mut g2, &v).unwrap().is_some());
        // Bronzong's lock is over events: evolving from the hand (Rare Candy too), not putting onto the Bench.
        use crate::spec::event::{EventKind, EventPred, RulesZone};
        static EVOLVE: LockDecl = LockDecl::on(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::Source(RulesZone::Hand)]), "BLOCKED_BY_EFFECT");
        let noctowl = hand(&g, me, "Noctowl PRE 78");
        assert_eq!(event_lock(&mut g, me, noctowl, true), None);
        apply_play_lock(&mut g.st.players[me], &EVOLVE, 1, 0);
        assert_eq!(event_lock(&mut g, me, noctowl, true), Some("BLOCKED_BY_EFFECT"));
        assert_eq!(event_lock(&mut g, me, noctowl, false), None, "an EnterPlay isn't evolving");
        assert_eq!(event_lock(&mut g, 1 - me, noctowl, true), None, "the opponent of the locked player isn't stopped");
    }

    #[test]
    fn jellicent_ex_locks_items_and_tools_while_active() {
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "hand": ["Potion POR 83", "Sacred Charm PFL 93"]},
            "opp": {"reset": true, "active": ["Frillish WHT 44", "Jellicent ex WHT 45"]}}));
        let me = g.st.active_player as usize;
        let (potion, charm) = (hand(&g, me, "Potion POR 83"), hand(&g, me, "Sacred Charm PFL 93"));
        assert_eq!(play_locked(&mut g, me, potion), Some("BLOCKED_BY_ABILITY"));
        // A Tool from the hand: the Attach event from the hand, whatever attaches it (id25, id230).
        assert_eq!(attach_lock(&mut g, me, charm, rule(me)), Some("BLOCKED_BY_ABILITY"));
        assert_eq!(attach_lock(&mut g, me, charm, ability(me)), Some("BLOCKED_BY_ABILITY"));
        // A Tool put on by an effect from the deck isn't played from the hand.
        let from_deck = in_deck(&g, me, "Sacred Charm PFL 93");
        assert_eq!(attach_lock(&mut g, me, from_deck, ability(me)), None);
        // Its owner is not stopped.
        assert_eq!(play_locked(&mut g, 1 - me, potion), None);
        // On the Bench it doesn't lock.
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "hand": ["Potion POR 83"]},
            "opp": {"reset": true, "active": "Duraludon PRE 69", "bench": [{"card": ["Frillish WHT 44", "Jellicent ex WHT 45"]}]}}));
        let potion = hand(&g, me, "Potion POR 83");
        assert_eq!(play_locked(&mut g, me, potion), None);
    }

    #[test]
    fn jellicent_ex_locked_by_iron_thorns_ex_lets_items_through() {
        let mut g = game(json!({"me": {"reset": true, "active": "Iron Thorns ex PRE 32", "hand": ["Potion POR 83"]},
            "opp": {"reset": true, "active": ["Frillish WHT 44", "Jellicent ex WHT 45"]}}));
        let me = g.st.active_player as usize;
        let potion = hand(&g, me, "Potion POR 83");
        let jellicent = active(&g, 1 - me);
        assert_eq!(ability_off(&g, jellicent), Some(true));
        assert_eq!(play_locked(&mut g, me, potion), None);
        // The same with Flutter Mane Active (mine): Midnight Fluttering turns it off.
        let mut g = game(json!({"me": {"reset": true, "active": "Flutter Mane PRE 43", "hand": ["Potion POR 83"]},
            "opp": {"reset": true, "active": ["Frillish WHT 44", "Jellicent ex WHT 45"]}}));
        let potion = hand(&g, me, "Potion POR 83");
        assert_eq!(ability_off(&g, active(&g, 1 - me)), Some(true));
        assert_eq!(play_locked(&mut g, me, potion), None);
    }

    #[test]
    fn arbok_judges_a_hand_card_by_its_printed_data() {
        // With Watchtower in play (id2147) a [C] Pokémon with an Ability still can't be played.
        let mut g = game(json!({"me": {"reset": true, "active": "Hoothoot PRE 77", "hand": ["Noctowl PRE 78", "Duraludon PRE 69", "Team Rocket's Arbok DRI 113", "Antique Root Fossil SCR 130"], "stadium": "Team Rocket's Watchtower ASC 210"},
            "opp": {"reset": true, "active": ["Team Rocket's Ekans DRI 112", "Team Rocket's Arbok DRI 113"]}}));
        let me = g.st.active_player as usize;
        let (noctowl, dura, arbok) = (hand(&g, me, "Noctowl PRE 78"), hand(&g, me, "Duraludon PRE 69"), hand(&g, me, "Team Rocket's Arbok DRI 113"));
        assert_eq!(ability_off(&g, noctowl), Some(false), "printed data in the hand");
        assert_eq!(event_lock(&mut g, me, noctowl, true), Some("BLOCKED_BY_ABILITY"), "evolving from the hand, Rare Candy too (id1133, id285, id1998)");
        assert_eq!(event_lock(&mut g, me, noctowl, false), Some("BLOCKED_BY_ABILITY"), "put onto the Bench from the hand");
        let fossil = hand(&g, me, "Antique Root Fossil SCR 130");
        assert_eq!(event_lock(&mut g, me, fossil, false), Some("BLOCKED_BY_ABILITY"), "a Fossil with an Ability is played as a Pokémon");
        assert_eq!(event_lock(&mut g, me, dura, false), None, "no Ability");
        assert_eq!(event_lock(&mut g, me, arbok, false), None, "Team Rocket's Pokémon are exempt");
        assert_eq!(event_lock(&mut g, me, arbok, true), None);
        // Its owner is not stopped.
        assert_eq!(event_lock(&mut g, 1 - me, noctowl, true), None);
    }

    #[test]
    fn arbok_with_iron_thorns_ex_active_still_blocks_a_rule_box_hand_card() {
        let mut g = game(json!({"me": {"reset": true, "active": "Iron Thorns ex PRE 32", "hand": ["Meowth ex POR 62"]},
            "opp": {"reset": true, "active": ["Team Rocket's Ekans DRI 112", "Team Rocket's Arbok DRI 113"]}}));
        let me = g.st.active_player as usize;
        let meowth = hand(&g, me, "Meowth ex POR 62");
        assert_eq!(event_lock(&mut g, me, meowth, false), Some("BLOCKED_BY_ABILITY"));
        // With Flutter Mane Active (mine) Arbok's Ability is off.
        let mut g = game(json!({"me": {"reset": true, "active": "Flutter Mane PRE 43", "hand": ["Meowth ex POR 62"]},
            "opp": {"reset": true, "active": ["Team Rocket's Ekans DRI 112", "Team Rocket's Arbok DRI 113"]}}));
        let meowth = hand(&g, me, "Meowth ex POR 62");
        assert_eq!(ability_off(&g, active(&g, 1 - me)), Some(true));
        assert_eq!(event_lock(&mut g, me, meowth, false), None);
    }

    #[test]
    fn genesect_locks_ace_spec_while_it_has_a_tool() {
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "hand": ["Enriching Energy SSP 191", "Potion POR 83"]},
            "opp": {"reset": true, "active": "Duraludon PRE 69", "bench": [{"card": "Genesect SFA 40", "tool": "Sacred Charm PFL 93"}]}}));
        let me = g.st.active_player as usize;
        let (charm, potion) = (hand(&g, me, "Enriching Energy SSP 191"), hand(&g, me, "Potion POR 83"));
        let _ = bench(&g, 1 - me, 0);
        assert_eq!(attach_lock(&mut g, me, charm, rule(me)), Some("BLOCKED_BY_EFFECT"));
        // An Ability attaching it from the hand is stopped too (id25, id230; RULES.md, decided 2026-10-09).
        assert_eq!(attach_lock(&mut g, me, charm, ability(me)), Some("BLOCKED_BY_EFFECT"));
        assert_eq!(play_locked(&mut g, me, potion), None, "not an ACE SPEC card");
        // Genesect without a Tool doesn't lock.
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "hand": ["Enriching Energy SSP 191"]},
            "opp": {"reset": true, "active": "Genesect SFA 40"}}));
        let charm = hand(&g, me, "Enriching Energy SSP 191");
        assert_eq!(attach_lock(&mut g, me, charm, rule(me)), None);
        // Flutter Mane Active (mine) turns Genesect's Ability off.
        let mut g = game(json!({"me": {"reset": true, "active": "Flutter Mane PRE 43", "hand": ["Enriching Energy SSP 191"]},
            "opp": {"reset": true, "active": "Genesect SFA 40", "active_tool": "Sacred Charm PFL 93"}}));
        let charm = hand(&g, me, "Enriching Energy SSP 191");
        assert_eq!(attach_lock(&mut g, me, charm, rule(me)), None);
    }

    #[test]
    fn antique_fossil_cant_retreat_and_palafin_cant_evolve_into() {
        let mut g = game(json!({"me": {"reset": true, "active": "Antique Root Fossil SCR 130", "bench": [{"card": "Duraludon PRE 69"}]},
            "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let me = g.st.active_player as usize;
        let fossil = active(&g, me);
        let _ = fossil;
        let retreat = |g: &mut Game| {
            let bench = g.st.players[me].bench.as_slice()[0];
            let c = crate::engine::change_active::ChangeActiveView::of(g, me, Some(bench), crate::spec::event::ActiveChange::Retreat, crate::cause::Cause::rule(crate::cause::RuleWhich::Retreat, me as u8));
            let v = crate::engine::change_active::view(g, &c);
            crate::engine::change_active::check_with(g, &v).unwrap()
        };
        let switch = |g: &mut Game| {
            let bench = g.st.players[me].bench.as_slice()[0];
            let c = crate::engine::change_active::ChangeActiveView::of(g, me, Some(bench), crate::spec::event::ActiveChange::Switch, crate::cause::Cause::new(crate::cause::CauseKind::Trainer, None, me as u8));
            crate::engine::change_active::refused(g, &c).unwrap()
        };
        assert_eq!(retreat(&mut g), Some("CANNOT_RETREAT"));
        assert!(!switch(&mut g), "it can still be switched (APR C-03)");
        // A Pokémon with no such lock retreats; the Fossil on the Bench isn't "the Active Pokémon".
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "bench": [{"card": "Antique Root Fossil SCR 130"}]},
            "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        assert_eq!(retreat(&mut g), None);
        // Palafin ex can't be put into play by evolving or otherwise, whoever does it; Palafin can.
        let mut g = game(json!({"me": {"reset": true, "active": "Finizen TWM 59", "hand": ["Palafin ex PRE 151", "Palafin TWM 60"]},
            "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let (ex, plain) = (hand(&g, me, "Palafin ex PRE 151"), hand(&g, me, "Palafin TWM 60"));
        assert_eq!(event_lock(&mut g, me, ex, true), Some("CANNOT_EVOLVE"));
        assert_eq!(event_lock(&mut g, 1 - me, ex, true), Some("CANNOT_EVOLVE"));
        assert_eq!(event_lock(&mut g, me, plain, true), None);
        assert_eq!(event_lock(&mut g, me, ex, false), Some("CANNOT_EVOLVE"), "every EnterPlay of the card too (RULES.md Evolution timing)");
    }

    #[test]
    fn a_stadium_without_use_text_cant_be_used() {
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "stadium": "Team Rocket's Watchtower ASC 210"},
            "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let me = g.st.active_player as usize;
        g.st.players[me].stadium_used_turn = -1;
        g.st.players[1 - me].stadium_used_turn = -1;
        assert_eq!(crate::engine::turn::can_use_stadium(&g, me), Err(crate::game::GameError("CANNOT_USE_STADIUM")));
        assert_eq!(crate::engine::turn::can_use_stadium(&g, 1 - me), Err(crate::game::GameError("CANNOT_USE_STADIUM")));
    }

    #[test]
    fn ability_off_from_the_stored_lock_state() {
        let g = game(json!({"me": {"reset": true, "active": "Flutter Mane PRE 43", "bench": [{"card": "Hoothoot PRE 77"}], "hand": ["Noctowl PRE 78"]},
            "opp": {"reset": true, "active": "Hoothoot PRE 77", "bench": [{"card": "Hoothoot PRE 77"}]}}));
        let me = g.st.active_player as usize;
        assert_eq!(ability_off(&g, active(&g, 1 - me)), Some(true), "Midnight Fluttering: the opponent's Active");
        assert_eq!(ability_off(&g, bench(&g, 1 - me, 0)), Some(false), "a Benched Pokémon keeps it");
        assert_eq!(ability_off(&g, bench(&g, me, 0)), Some(false), "so does my own");
        assert_eq!(ability_off(&g, hand(&g, me, "Noctowl PRE 78")), Some(false), "a hand card has its printed Ability");
        // Watchtower: every [C] Pokémon in play.
        let g = game(json!({"me": {"reset": true, "active": "Hoothoot PRE 77", "bench": [{"card": "Duraludon PRE 69"}], "stadium": "Team Rocket's Watchtower ASC 210"},
            "opp": {"reset": true, "active": "Hoothoot PRE 77"}}));
        assert_eq!(ability_off(&g, active(&g, me)), Some(true));
        assert_eq!(ability_off(&g, active(&g, 1 - me)), Some(true));
        assert_eq!(ability_off(&g, bench(&g, me, 0)), Some(false), "a Metal Pokémon");
    }
}

#[cfg(test)]
mod event_lock_marker_tests {
    //! A lock over an event is consulted only when its card's mask carries the marker `event_locked` and
    //! `may_lock_event` read (`lock_marker`), which `block_kinds` sets from the effect kinds of `forbids`. The two
    //! must agree for every kind, or a lock (a batch 4 lock or Prevent over GainCondition, say) would silently
    //! never be asked.
    use super::*;
    use crate::spec::event::{EventKind as E, EventPred};

    const ALL: &[E] = &[
        E::EnterPlay,
        E::Evolve,
        E::Devolve,
        E::Swap,
        E::Attach,
        E::MoveEnergy,
        E::MoveTool,
        E::PlayTrainer,
        E::ChangeActive,
        E::Damage,
        E::PlaceCounters,
        E::MoveCounters,
        E::RemoveCounters,
        E::GainCondition,
        E::RemoveCondition,
        E::KnockOut,
        E::TakePrizes,
        E::Discard,
        E::Draw,
        E::PutIntoHand,
        E::PutIntoDeck,
        E::LeavePlay,
        E::Shuffle,
        E::Look,
        E::Reveal,
        E::CoinFlip,
        E::ApplyEffect,
        E::StateCheck,
        E::GameEnd,
        E::Mulligan,
        E::SetPrizes,
        E::BeginTurn,
        E::EndTurn,
        E::Checkup,
        E::UseAttack,
        E::UseAbility,
        E::UseStadium,
        E::Retreat,
    ];

    /// Exhaustive (no wildcard): a new kind doesn't compile until it is listed in `ALL`.
    fn listed(k: E) -> bool {
        match k {
            E::EnterPlay | E::Evolve | E::Devolve | E::Swap | E::Attach | E::MoveEnergy | E::MoveTool | E::PlayTrainer | E::ChangeActive | E::Damage => ALL.contains(&k),
            E::PlaceCounters | E::MoveCounters | E::RemoveCounters | E::GainCondition | E::RemoveCondition | E::KnockOut | E::TakePrizes | E::Discard => ALL.contains(&k),
            E::Draw | E::PutIntoHand | E::PutIntoDeck | E::LeavePlay | E::Shuffle | E::Look | E::Reveal | E::CoinFlip | E::ApplyEffect | E::StateCheck | E::GameEnd => ALL.contains(&k),
            E::Mulligan | E::SetPrizes | E::BeginTurn | E::EndTurn | E::Checkup | E::UseAttack | E::UseAbility | E::UseStadium | E::Retreat => ALL.contains(&k),
        }
    }

    #[test]
    fn every_event_kind_has_a_marker_block_kinds_sets() {
        for &k in ALL {
            assert!(listed(k));
            static ERR: &str = "BLOCKED_BY_EFFECT";
            let m = block_kinds(&LockDecl::on(EventPred::Kind(k), ERR));
            match (k.effect_kind(), lock_marker(k)) {
                (Some(x), Some(marker)) => {
                    assert!(m.has(x), "{k:?}: the lock isn't listed under its effect kind");
                    assert!(m.has(marker), "{k:?}: block_kinds doesn't set the marker event_locked reads");
                }
                // No lock over it is consulted (no text forbids it: Damage, KnockOut, ...): a lock over it lists nothing.
                (_, None) => assert_eq!(m, KindMask::EMPTY, "{k:?}"),
                (None, Some(marker)) => panic!("{k:?}: marker {marker:?} but no effect kind"),
            }
        }
    }

    /// Every lock a card declares over events names a kind an effect carries (else it is never consulted).
    #[test]
    fn declared_event_locks_are_consulted() {
        for s in crate::cards::registry::SPECS.iter() {
            for p in s.passives {
                if let Modifier::BlockUse(b) = &p.modifier {
                    if !b.lock.forbids.is_never() {
                        assert!(block_kinds(&b.lock) != KindMask::EMPTY, "{}: a lock over no consulted event kind", s.class);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod prevent_marker_tests {
    //! A `Prevent` over events is consulted only when its card's mask carries the marker `event_prevented` reads
    //! (`prevent_marker`), which `prevent_kinds` sets from the effect kinds of `from`; and only the routines of the
    //! kinds with a marker ask the reader. A prevention over GainCondition alone (Slowpoke's Dopey Face) must be
    //! asked; one over a kind whose routine doesn't ask would silently never be.
    use super::*;
    use crate::spec::event::{EventKind as E, EventPred};

    const ALL: &[E] = &[
        E::EnterPlay, E::Evolve, E::Devolve, E::Swap, E::Attach, E::MoveEnergy, E::MoveTool, E::PlayTrainer, E::ChangeActive,
        E::Damage, E::PlaceCounters, E::MoveCounters, E::RemoveCounters, E::GainCondition, E::RemoveCondition, E::KnockOut,
        E::TakePrizes, E::Discard, E::Draw, E::PutIntoHand, E::PutIntoDeck, E::LeavePlay, E::Shuffle, E::Look, E::Reveal,
        E::CoinFlip, E::ApplyEffect, E::StateCheck, E::GameEnd, E::Mulligan, E::SetPrizes, E::BeginTurn, E::EndTurn, E::Checkup,
        E::UseAttack, E::UseAbility, E::UseStadium, E::Retreat,
    ];

    #[test]
    fn every_prevented_kind_sets_the_marker_the_reader_reads() {
        for &k in ALL {
            let p = PreventSpec::on(SlotPred::Any, EventPred::Kind(k));
            let m = prevent_kinds(&p);
            if let Some(marker) = prevent_marker(k) {
                let x = k.effect_kind().expect("a prevented kind has an effect kind");
                assert!(m.has(x), "{k:?}: the prevention isn't listed under its effect kind");
                assert!(m.has(marker), "{k:?}: prevent_kinds doesn't set the marker event_prevented reads");
            }
        }
        // The four events batch 4 kinds are consulted.
        for k in [E::GainCondition, E::RemoveCondition, E::RemoveCounters, E::CoinFlip] {
            assert!(prevent_marker(k).is_some(), "{k:?}");
        }
    }

    /// Every `Prevent` a card declares is consulted somewhere, and the kinds it names explicitly (beyond "every effect",
    /// which a cause-only declaration ranges over by design) are kinds whose routine asks the reader or a B6-OLD probe
    /// stands for.
    #[test]
    fn declared_preventions_are_consulted() {
        let mut consulted = KindMask::EMPTY;
        for &k in ALL {
            if let (Some(_), Some(x)) = (prevent_marker(k), k.effect_kind()) {
                consulted = consulted.or(crate::effects::mask(&[x]));
            }
        }
        let effects = crate::spec::event::EFFECT_EVENT_KINDS;
        let mut n = 0;
        for s in crate::cards::registry::SPECS.iter() {
            for p in s.passives {
                if let Modifier::Prevent(pr) = &p.modifier {
                    if pr.from.is_never() {
                        continue;
                    }
                    n += 1;
                    assert!(prevent_kinds(pr) != KindMask::EMPTY, "{}: a prevention no routine consults", s.class);
                    let kinds = pr.from.prevent_kinds();
                    let explicit = KindMask([kinds.0[0] & !effects.0[0], kinds.0[1] & !effects.0[1], kinds.0[2] & !effects.0[2], kinds.0[3] & !effects.0[3]]);
                    let named_effects = if and(kinds, effects) == effects { KindMask::EMPTY } else { kinds };
                    let named = explicit.or(named_effects);
                    let outside = KindMask([named.0[0] & !consulted.0[0], named.0[1] & !consulted.0[1], named.0[2] & !consulted.0[2], named.0[3] & !consulted.0[3]]);
                    assert_eq!(outside, KindMask::EMPTY, "{}: a prevention over a kind whose routine doesn't ask event_prevented", s.class);
                }
            }
        }
        // Batch 4: Slowpoke, Hoothoot, the two Antique Fossils, Bubbly Water Energy, Festival Grounds, Yveltal; batch 6 (one
        // declaration over the cause each, the batch 5 ChangeActive ones merged into it): Mist Energy, Rocky Fighting Energy,
        // Skeledirge, Team Rocket's Articuno, Empoleon ex, Milotic ex, Rabsca, Acerola's Mischief, Antique Cover Fossil, the
        // four Hide 'n' Sneak Pokémon, Battle Cage; over Damage: the Tera rule (21 cards), Sylveon, Crustle DRI, Shaymin,
        // Neutralization Zone, Farigiraf ex, Cornerstone Mask Ogerpon ex, Shadowy Darkness Energy, Fezandipiti (coin); over
        // LeavePlay: Milotic TWM 50 (closeout).
        assert_eq!(n, 51);
    }

    /// "Prevent all effects of attacks" never prevents Damage; "damage from and effects of" does, by the same cause.
    #[test]
    fn effects_declarations_and_damage() {
        assert!(!ranges(&EFFECTS_OF_OPP_ATTACKS, E::Damage));
        assert!(ranges(&EFFECTS_OF_OPP_ATTACKS, E::GainCondition) && ranges(&EFFECTS_OF_OPP_ATTACKS, E::PlaceCounters) && ranges(&EFFECTS_OF_OPP_ATTACKS, E::LeavePlay));
        let both = EventPred::All(&[DAMAGE_OR_EFFECTS, EFFECTS_OF_OPP_ATTACKS]);
        assert!(ranges(&both, E::Damage) && ranges(&both, E::ChangeActive) && ranges(&both, E::LeavePlay));
        assert!(!ranges(&EventPred::Kind(E::Damage), E::LeavePlay));
        // Cards from the hand or the deck aren't done to a Pokémon (user decision D1): no cause-only prevention ranges
        // over Discard, Draw, PutIntoHand or PutIntoDeck.
        for k in [E::Discard, E::Draw, E::PutIntoHand, E::PutIntoDeck] {
            assert!(!ranges(&EFFECTS_OF_OPP_ATTACKS, k) && !ranges(&both, k), "{k:?}");
        }
    }

    /// The preventions that make an affected Pokémon recover when they come into force (id289): the cause-free ones
    /// over GainCondition (Slowpoke, Hoothoot, the Fossils, Bubbly Water Energy, Festival Grounds), not Yveltal's
    /// "can't be healed", and not one that depends on the cause.
    #[test]
    fn which_preventions_recover() {
        let mut recovering: Vec<&str> = Vec::new();
        for s in crate::cards::registry::SPECS.iter() {
            for p in s.passives {
                if let Modifier::Prevent(pr) = &p.modifier {
                    if prevention_recovers(pr) {
                        assert!(s.card_impl().mask.has(crate::effects::k::CHECK_TABLE_STATE), "{}: the recovery is never dispatched", s.class);
                        recovering.push(s.class);
                    }
                }
            }
        }
        recovering.sort();
        assert_eq!(recovering.len(), 6, "{recovering:?}");
        assert!(!recovering.iter().any(|c| c.starts_with("Yveltal")));
        let by_attacks = PreventSpec::on(
            SlotPred::Holder,
            EventPred::All(&[EventPred::Kind(E::GainCondition), EventPred::Cause(crate::spec::event::CausePred::Kind(crate::cause::CauseKind::Attack))]),
        );
        assert!(!prevention_recovers(&by_attacks), "a prevention that depends on the cause removes no existing effect");
    }
}

#[cfg(test)]
mod attached_lock_tests {
    //! The gate of `lock_sync` after the attaching events (`lock_sync_attached`) assumes that a lock source's
    //! Ability is only turned off by declarations it accounts for: the Ability locks (their spot predicates and
    //! probes: `lock_reads_attached`) and the source's own Ability (`CardSpec::mask`). A card declaring another
    //! handler of the Power probe or of the Ability-effect probe would need the marker too.
    use super::*;
    use crate::effects::k;

    #[test]
    fn power_probe_handlers_are_accounted_for() {
        for s in crate::cards::registry::SPECS.iter() {
            let m = s.card_impl().mask;
            let lock = declares_ability_lock(s.passives);
            if m.has(k::POWER) {
                assert!(!s.powers.is_empty() || lock, "{}: a Power handler that is neither an Ability nor an Ability lock", s.class);
            }
            if m.has(k::EFFECT_OF_ABILITY) {
                assert!(
                    s.passives.iter().any(|p| matches!(p.modifier, Modifier::ActiveLock(ActiveLock::Initialization))),
                    "{}: an Ability-effect handler other than Initialization",
                    s.class
                );
            }
        }
    }

    #[test]
    fn which_locks_read_attached_cards() {
        let marked: Vec<&str> =
            crate::cards::registry::SPECS.iter().filter(|s| s.card_impl().mask.has(k::DECLARES_ATTACHED_LOCK)).map(|s| s.class).collect();
        // Team Rocket's Watchtower: "Colorless Pokémon" is the type as the game checks it (attached cards can
        // change it), and its probe is the Stadium's effect on the spot.
        assert_eq!(marked, vec!["TeamRocketsWatchtower"]);
    }
}
