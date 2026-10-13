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
    /// "You may ignore all results of those coin flips and begin flipping those coins again" (events batch 7, user
    /// decision D7; Backtrack Badge): offered before the results of the flips its event predicate matches are used.
    Reflip(ReflipSpec),
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
    /// "This Pokémon may use an attack it has twice" while `guard` holds (Festival Lead: "If Festival Grounds is in play";
    /// batch 8 C12c): read by the core when the second use would be offered (`attack::attack_twice`), from the Pokémon
    /// that used the attack, for an attack printed on it.
    AttackTwice(AttackTwiceSpec),
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
}

/// The Active-Spot Ability locks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveLock {
    /// Flutter Mane's Midnight Fluttering: your opponent's Active Pokémon has no Abilities, except for
    /// Midnight Fluttering (a `Prevent` over continuous effects of Abilities, Hide 'n' Sneak, takes precedence: C11).
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
    /// The damage to the Pokémon, before Weakness and Resistance (the Damage routine's step 2).
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
            NonStack::CurlyWall => CURLY_WALL,
        }
    }
}

/// `DamageAcc::flags`: Bouffalant's Curly Wall already reduced this damage.
const CURLY_WALL: u8 = 1 << 1;

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
    m
}

const fn and(a: KindMask, b: KindMask) -> KindMask {
    KindMask([a.0[0] & b.0[0], a.0[1] & b.0[1], a.0[2] & b.0[2], a.0[3] & b.0[3]])
}

/// The event families whose routine asks the `Prevent` reader (`event_prevented`), each with the marker a prevention over
/// it sets (`prevent_kinds`) and the reader tests (`prevent_marker`).
const PREVENT_FAMILIES: [(KindMask, u32); 12] = [
    (super::event::CONTINUOUS_EVENT_KINDS, crate::effects::k::DECLARES_CONTINUOUS_PREVENT),
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
const LOCK_FAMILIES: [(KindMask, u32); 11] = [
    (super::event::USE_EVENT_KINDS, crate::effects::k::DECLARES_USE_LOCK),
    (super::event::PLAY_EVENT_KINDS, crate::effects::k::DECLARES_PLAY_LOCK),
    (super::event::CARD_EVENT_KINDS, crate::effects::k::DECLARES_CARD_LOCK),
    // A card leaving play into a zone (Milotic TWM 50: "can't be put into your opponent's hand"): the card locks' marker.
    (super::event::LEAVE_EVENT_KINDS, crate::effects::k::DECLARES_CARD_LOCK),
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
#[derive(Clone, Copy)]
pub enum LockWhile {
    /// The source is its owner's Active Pokémon.
    Active,
    /// The source's Pokémon has a Pokémon Tool attached.
    HasTool,
    /// The card the action uses is the source itself (the Pokémon evolving into it, the Active Pokémon
    /// that retreats, the Stadium being used).
    CardIsSource,
    /// "... unless <condition>": the lock holds while the condition (read for the source's owner) doesn't (Team Rocket's
    /// Mewtwo ex's Power Saver, Slaking ex's Born to Slack: "this Pokémon can't attack unless ...").
    Unless(&'static Cond),
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
/// attack leaves on the opponent (`Lasting::Lock` on the opponent, stored in the lasting table as a
/// [`crate::state::LastingWhat::PlayerLock`]) are this one declaration. Events batch 7: the play locks are over PlayTrainer ("your
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

/// "This Pokémon can't attack" / "can't use attacks" / "can't use [Attack Name]" for a time (APR C-15; events batch 7, user
/// decision D14): a lock over the UseAttack of the Pokémon it is on, left by an attack (`Lasting::Lock`; the attack it
/// names is the stored lock's, `LastingWhat::Lock::attack`). The player can't announce nor use the attack.
pub static CANT_ATTACK: LockDecl = LockDecl::on(
    super::event::EventPred::All(&[super::event::EventPred::Kind(super::event::EventKind::UseAttack), super::event::EventPred::This(crate::spec::prelude::Role::Card)]),
    "BLOCKED_BY_EFFECT",
);

/// "This Pokémon can't use [this attack] again until it leaves the Active Spot" (Gouging Fire ex's Blaze Blitz): the same
/// lock, named, until the Pokémon's effects of attacks end.
pub static CANT_USE_ATTACK_AGAIN: LockDecl = LockDecl::on(
    super::event::EventPred::All(&[super::event::EventPred::Kind(super::event::EventKind::UseAttack), super::event::EventPred::This(crate::spec::prelude::Role::Card)]),
    "CANNOT_USE_ATTACK",
);

/// "This Pokémon can't retreat" / "the Defending Pokémon can't retreat" for a time (events batch 7, user decision D14): a
/// lock over the ChangeActive of a retreat whose leaving Pokémon is the one it is on. Retreating only: it can still be
/// switched (APR C-03).
pub static CANT_RETREAT: LockDecl = LockDecl::on(
    super::event::EventPred::All(&[
        super::event::EventPred::Kind(super::event::EventKind::ChangeActive),
        super::event::EventPred::Change(super::event::ActiveChange::Retreat),
        super::event::EventPred::From(SlotPred::IsThisPokemon),
    ]),
    "BLOCKED_BY_EFFECT",
);

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
#[derive(Debug)]
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

/// `Modifier::AttackTwice`.
pub struct AttackTwiceSpec {
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
        // The derived layer's modifiers are read through the declaration index (`decl::fam`); nothing is dispatched to them.
        Modifier::HpBonus(_) | Modifier::HpMod(_) => KindMask::EMPTY,
        // The Deal stage, the defender-side modifiers and the survive-on-10 replacements are read by the Damage routine's
        // calculation through the declaration index (`decl::fam::DAMAGE_DEALT`, `DAMAGE_TAKEN`).
        Modifier::DamageDealt(d) => match d.stage {
            DamageStage::Deal => KindMask::EMPTY,
            DamageStage::Attack => mask(&[k::ATTACK]),
        },
        Modifier::DamageTaken(_) | Modifier::SurviveOnTen(_) => KindMask::EMPTY,
        Modifier::CheckupDamage(_) => mask(&[k::BETWEEN_TURNS]),
        Modifier::NextTurnBonus(_) => mask(&[k::ATTACK]),
        Modifier::BenchAttacks(_) => KindMask::EMPTY,
        // The first-turn permission is read by the UseAttack's checks (`grants_first_turn_attack`).
        Modifier::AttackFlags(a) => {
            if a.first_turn {
                KindMask::EMPTY
            } else {
                mask(&[k::ATTACK])
            }
        }
        Modifier::BlockUse(b) => block_kinds(&b.lock),
        Modifier::Reflip(_) => mask(&[k::COIN_FLIP, k::DECLARES_REFLIP]),
        Modifier::ProvidesEnergy(_) | Modifier::ProvidesEnergyBoost(_) => KindMask::EMPTY,
        // The attach refusal is a check of the Attach routine (`engine::attach::check_attach_with`); the discard and the
        // recovery are the state check's (`knockout::table_state`, through the declaration index).
        Modifier::AttachGuard(_) | Modifier::Recover(_) => KindMask::EMPTY,
        Modifier::AbilityLock(l) => {
            if lock_reads_attached(l) {
                mask(&[k::DECLARES_ABILITY_LOCK, k::POWER, k::DECLARES_ATTACHED_LOCK])
            } else {
                mask(&[k::DECLARES_ABILITY_LOCK, k::POWER])
            }
        }
        Modifier::Prevent(p) => prevent_kinds(p).or(match p.what {
            PreventWhat::None => KindMask::EMPTY,
            PreventWhat::ToolEffects => KindMask::EMPTY,
        }),
        Modifier::PrizeAdjust(_) | Modifier::PrizeAdjustOnce(_) => mask(&[k::KNOCK_OUT]),
        Modifier::TypeOverride(_) => KindMask::EMPTY,
        Modifier::WeaknessOverride(_) => KindMask::EMPTY,
        Modifier::ActiveLock(ActiveLock::MidnightFluttering) => mask(&[k::DECLARES_ABILITY_LOCK, k::POWER]),
        Modifier::ActiveLock(ActiveLock::Initialization) => mask(&[k::DECLARES_ABILITY_LOCK, k::POWER]),
        Modifier::Permit(pm) => {
            let mut m = mask(&[k::DECLARES_PERMIT]);
            let mut i = 0;
            while i < pm.lifts.len() {
                m = crate::spec::with(m, crate::engine::enter::limit_kind(pm.lifts[i]));
                i += 1;
            }
            m
        }
        Modifier::GrantAttacks(_) => KindMask::EMPTY,
        Modifier::AttackCost(_) => KindMask::EMPTY,
        Modifier::RetreatCost(_) => KindMask::EMPTY,
        Modifier::BenchSize(_) => KindMask::EMPTY,
        // Read by the core when a second attack would be offered (`attack::attack_twice`).
        Modifier::AttackTwice(_) => KindMask::EMPTY,
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

fn slot_where(st: &crate::state::State, f: impl Fn(&crate::state::Slot, usize, u8) -> bool) -> Option<SlotRef> {
    // The Pokémon in play, Active first (`Player::in_play`), without building the list.
    for p in 0..2 {
        let pl = &st.players[p];
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
    if g.reach_skip != 0 && origin == RuleSource::Ability && g.reach_skip & (1u128 << me) != 0 {
        // VERIFY_LEGAL's copy without the Abilities that don't reach the checked Pokémon (`derived::verify_reach`).
        return None;
    }
    locate_st(&g.st, me, origin)
}

/// [`locate`] over the state.
pub(crate) fn locate_st(st: &crate::state::State, me: CardId, origin: RuleSource) -> Option<Located> {
    let r = locate_at(st, me, origin);
    if crate::game::verify_cache() {
        let slow = locate_slow(st, me, origin);
        assert_eq!(r.map(|l| (l.owner, l.held)), slow.map(|l| (l.owner, l.held)), "passive::locate differs from the scan for card {me} ({origin:?})");
    }
    r
}

/// [`locate`] from the card's location (`State::locate`, `State::tool_spot`): no walk of the board.
#[inline]
fn locate_at(st: &crate::state::State, me: CardId, origin: RuleSource) -> Option<Located> {
    match origin {
        RuleSource::Tool => st.tool_spot(me).map(|(p, s)| Located { owner: p, held: Some(SlotRef::new(p, s)) }),
        RuleSource::Energy | RuleSource::Ability => {
            // A card in a Pokémon's spot that is in play (`State::locate` finds only those).
            let Some(ListRef::Slot(p, s)) = st.locate(me) else { return None };
            let (p, sl) = (p as usize, &st.players[p as usize].slots[s as usize]);
            let ok = match origin {
                RuleSource::Energy => !sl.tools.contains(me),
                _ => st.slot_pokemon(p, s) == Some(me),
            };
            ok.then(|| Located { owner: p, held: Some(SlotRef::new(p, s)) })
        }
        RuleSource::Stadium => {
            if st.stadium_card() == Some(me) {
                let owner = st.locate(me).and_then(|l| l.owner()).unwrap_or_else(|| st.owner(me));
                Some(Located { owner, held: None })
            } else {
                None
            }
        }
        RuleSource::TrainerEffect | RuleSource::CardRule => {
            let owner = st.locate(me).and_then(|l| l.owner()).unwrap_or_else(|| st.owner(me));
            Some(Located { owner, held: None })
        }
    }
}

/// [`locate`] the slow way (`PTCG_VERIFY_CACHE=1` checks every answer against it).
fn locate_slow(st: &crate::state::State, me: CardId, origin: RuleSource) -> Option<Located> {
    match origin {
        RuleSource::Tool => slot_where(st, |sl, _, _| sl.tools.contains(me)).map(|s| Located { owner: s.p as usize, held: Some(s) }),
        RuleSource::Energy => slot_where(st, |sl, _, _| sl.cards.contains(me) && !sl.tools.contains(me)).map(|s| Located { owner: s.p as usize, held: Some(s) }),
        RuleSource::Ability => slot_where(st, |sl, p, s| sl.cards.contains(me) && st.slot_pokemon(p, s) == Some(me)).map(|s| Located { owner: s.p as usize, held: Some(s) }),
        RuleSource::Stadium => {
            if st.stadium_card() == Some(me) {
                let owner = st.locate_scan(me).and_then(|l| l.owner()).unwrap_or_else(|| st.owner(me));
                Some(Located { owner, held: None })
            } else {
                None
            }
        }
        RuleSource::TrainerEffect | RuleSource::CardRule => {
            let owner = st.locate_scan(me).and_then(|l| l.owner()).unwrap_or_else(|| st.owner(me));
            Some(Located { owner, held: None })
        }
    }
}

/// Is the card's effect off: the lock probe derived from its origin. `affected`
/// is the Pokémon the effect is about (a Stadium's probe is per Pokémon).
pub(crate) fn blocked<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, at: Located, affected: Option<SlotRef>) -> bool {
    match origin {
        RuleSource::Ability => g.has_no_ability(at.owner, me, None),
        RuleSource::Tool => g.tool_off(at.owner, me),
        RuleSource::Energy => match at.held {
            Some(_) => crate::derived::special_energy_off(g.g(), me),
            None => false,
        },
        RuleSource::Stadium => match affected {
            Some(_) => crate::derived::stadium_off(g.g()),
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

pub(crate) fn apply(g: &mut Game, me: CardId, e: EffId, ps: &'static Passive) -> R {
    match &ps.modifier {
        // The derived layer's modifiers (HP, types, Weakness, provided Energy, costs, the attack list) are read through the
        // declaration index (`derived::compute_*`); nothing is dispatched to them.
        Modifier::HpBonus(_) | Modifier::HpMod(_) => Ok(()),
        // Read by the Damage routine's calculation (`engine::damage::deal`); nothing is dispatched to it.
        Modifier::SurviveOnTen(_) => Ok(()),
        Modifier::CheckupDamage(c) => checkup_damage(g, me, e, ps.origin, c),
        Modifier::NextTurnBonus(b) => next_turn_bonus(g, me, e, b),
        Modifier::BenchAttacks(_) => Ok(()),
        Modifier::AttackFlags(a) => attack_flags(g, me, e, ps.origin, a),
        Modifier::DamageDealt(d) => damage_dealt(g, me, e, ps.origin, d),
        // Read by the Damage routine's calculation (`engine::damage::deal`); nothing is dispatched to it.
        Modifier::DamageTaken(_) => Ok(()),
        // Locks are read by the event's routine and legality (`event_locked`); nothing is dispatched to them.
        Modifier::BlockUse(_) => Ok(()),
        Modifier::ProvidesEnergy(_) | Modifier::ProvidesEnergyBoost(_) => Ok(()),
        // Read by the state check (`knockout::table_state`) and the Attach routine; nothing is dispatched to it.
        Modifier::AttachGuard(_) => Ok(()),
        Modifier::AttackCost(_) => Ok(()),
        // Read by the state check (`knockout::table_state`); nothing is dispatched to it.
        Modifier::Recover(_) => Ok(()),
        Modifier::AbilityLock(l) => ability_lock(g, me, e, l),
        Modifier::Prevent(p) => prevent(g, me, e, ps.origin, p),
        Modifier::TypeOverride(_) => Ok(()),
        // Permissions are read where a limit is checked (`engine::enter::permitted`); nothing is dispatched to them.
        Modifier::Permit(_) => Ok(()),
        // Re-flips are read where a card's flips are made (`engine::condition::flip_coin`); nothing is dispatched to them.
        Modifier::Reflip(_) => Ok(()),
        Modifier::ActiveLock(l) => active_lock(g, me, e, *l),
        Modifier::WeaknessOverride(_) => Ok(()),
        // The KnockOut routine reads the game's Prize adjustments (`knockout::prize_adjustments`); a copied attack's source
        // card, run for the copycat (`copy_attack::resolve_sessions`), still reaches its own here.
        Modifier::PrizeAdjust(d) => prize_adjust(g, me, e, ps.origin, d),
        Modifier::PrizeAdjustOnce(d) => prize_adjust_once(g, me, e, ps.origin, d),
        Modifier::GrantAttacks(_) | Modifier::RetreatCost(_) => Ok(()),
        // Read by the state check (`knockout::table_state`); nothing is dispatched to it.
        Modifier::BenchSize(_) => Ok(()),
        Modifier::AttackTwice(_) => Ok(()),
        _ => unimplemented!("spec passive not implemented yet (passive.rs)"),
    }
}

/// The `TypeOverride` of `me` on the Pokémon in `target`: the types it gets, when the declaration applies there (the
/// derived layer's `types`).
pub(crate) fn type_override<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, t: &TypeOverrideSpec, target: SlotRef) -> R<Option<&'static [CardType]>> {
    let Some(at) = locate(g.g(), me, origin) else { return Ok(None) };
    if !slot_pred_m(g, me, target, &t.subject)? || blocked(g, me, origin, at, Some(target)) || !ability_effect_reaches(g, me, origin, at.owner, target, false)? {
        return Ok(None);
    }
    Ok(Some(t.set))
}

/// The `WeaknessOverride` of `me` on the Pokémon in `target` (the derived layer's Weakness): the type it sets, when it
/// applies. The Ability's lock probe is made for the checked Pokémon's owner, first; then whether the effect of the
/// Ability reaches the Pokémon (`effect_of_ability_reaches`; the continuous-effect `Prevent` view replaces it in C11).
pub(crate) fn weakness_override<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, w: &WeaknessOverrideSpec, target: SlotRef) -> R<Option<CardType>> {
    let player = target.p as usize;
    if g.has_no_ability(player, me, None) {
        return Ok(None);
    }
    let Some(at) = locate(g.g(), me, origin) else { return Ok(None) };
    if at.owner == player || !slot_pred_m(g, me, target, &w.subject)? {
        return Ok(None);
    }
    if !ability_effect_reaches(g, me, origin, at.owner, target, false)? {
        return Ok(None);
    }
    Ok(effect_of_ability_reaches(g.g(), me, crate::effects::PowerRef { card: me, index: 0 }, target).then_some(w.weakness))
}

/// The HP `HpBonus` / `HpMod` of `me` gives the Pokémon in `target` (0 when it doesn't apply): the derived layer's `hp`
/// adds them up.
pub(crate) fn hp_mod_delta<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, n: i32, subject: &SlotPred, guard: &Cond, target: SlotRef) -> R<i32> {
    let Some(at) = locate(g.g(), me, origin) else { return Ok(0) };
    if !slot_pred_m(g, me, target, subject)? || blocked(g, me, origin, at, Some(target)) || !guard_ok_m(g, me, origin, at.owner, guard)? {
        return Ok(0);
    }
    if !ability_effect_reaches(g, me, origin, at.owner, target, false)? {
        return Ok(0);
    }
    Ok(n)
}

/// An attack's damage to one Pokémon being calculated (the Damage routine's steps 2 and 5, `engine::damage::deal`): what
/// the attacker-side and defender-side declarations read and change.
#[derive(Clone, Copy, Debug)]
pub struct DamageAcc {
    pub b: AtkBase,
    pub damage: i32,
    /// The damage was calculated in full (the attacker-side modifiers, and Weakness and Resistance when in an Active Spot).
    pub weakness_applied: bool,
    /// A survive-on-10 replacement of a full-HP Pokémon is armed.
    pub survive: bool,
    /// The non-stacking modifiers already applied (`fx_flag::DAMAGE_INCREASED`, `CURLY_WALL`).
    pub flags: u8,
}

/// `DamageDealt` at the Attack stage, over the `Attack` effect ("this attack does N more damage" before it is dealt).
fn damage_dealt(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &DamageDealtSpec) -> R {
    let (player, source, damage, attack) = match (d.stage, *g.e(e)) {
        (DamageStage::Attack, Effect::Attack { p, source, damage, attack, .. }) => (p as usize, source, damage, attack),
        _ => return Ok(()),
    };
    let (mut dmg, mut flags) = (damage, g.fx_flags(e));
    damage_dealt_core(g, me, origin, d, player, source, None, attack, &mut dmg, &mut flags)?;
    if let Effect::Attack { damage, .. } = g.e_mut(e) {
        *damage = dmg;
    }
    if flags & fx_flag::DAMAGE_INCREASED != 0 {
        g.set_fx_flag(e, fx_flag::DAMAGE_INCREASED);
    }
    Ok(())
}

/// `DamageDealt` at the Deal stage: the attacker-side modifier of `me` on the damage being calculated (step 2).
pub(crate) fn damage_dealt_deal(g: &mut Game, me: CardId, origin: RuleSource, d: &DamageDealtSpec, acc: &mut DamageAcc) -> R {
    if d.stage != DamageStage::Deal {
        return Ok(());
    }
    let b = acc.b;
    damage_dealt_core(g, me, origin, d, b.player as usize, b.source, Some(b.target), b.attack, &mut acc.damage, &mut acc.flags)
}

#[allow(clippy::too_many_arguments)]
fn damage_dealt_core(g: &mut Game, me: CardId, origin: RuleSource, d: &DamageDealtSpec, player: usize, source: SlotRef, target: Option<SlotRef>, attack: AttackRef, damage_now: &mut i32, flags: &mut u8) -> R {
    let damage = *damage_now;
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
    if d.nonstacking && *flags & fx_flag::DAMAGE_INCREASED != 0 {
        return Ok(());
    }
    // An opponent's Ability changing the damage the attacking Pokémon does: a continuous effect done to it.
    if !ability_effect_reaches(g, me, origin, at.owner, source, false)? {
        return Ok(());
    }
    *damage_now += d.amount;
    if d.nonstacking {
        *flags |= fx_flag::DAMAGE_INCREASED;
    }
    Ok(())
}

/// The preconditions both "takes less" and "prevent" share: damage put on a
/// Pokémon by an attack of the opponent's Pokémon, during the attack phase,
/// not ignored by Shred. Returns the attack base and the card's place.
#[allow(clippy::too_many_arguments)]
fn damage_taken_prelude(
    g: &mut Game,
    me: CardId,
    b: AtkBase,
    origin: RuleSource,
    side: Side,
    subject: &SlotPred,
    source: &SlotPred,
    guard: &Cond,
    from_any_attack: bool,
) -> R<Option<(AtkBase, Located)>> {
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
    if !ability_effect_reaches(g, me, origin, at.owner, t, false)? {
        return Ok(None);
    }
    Ok(Some((b, at)))
}

/// `DamageTaken` of `me`: the defender-side modifier on the damage being calculated (step 5).
pub(crate) fn damage_taken_into(g: &mut Game, me: CardId, origin: RuleSource, d: &DamageTakenSpec, acc: &mut DamageAcc) -> R {
    let Some((b, _)) = damage_taken_prelude(g, me, acc.b, origin, d.side, &d.subject, &d.source, &d.guard, d.from_any_attack)? else { return Ok(()) };
    if let Some(ns) = d.nonstacking {
        if acc.flags & ns.flag() != 0 {
            return Ok(());
        }
    }
    if d.then_discard {
        // "If the Pokémon is damaged": with no damage taken the Tool stays.
        let damage_now = acc.damage;
        if damage_now <= 0 || crate::engine::damage::would_be_prevented(g, &b, damage_now)? {
            return Ok(());
        }
    }
    acc.damage -= d.amount;
    if let Some(ns) = d.nonstacking {
        acc.flags |= ns.flag();
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

/// `ProvidesEnergy` of `me`: the entries it adds to the Energy map of the Pokémon in `source` (asked by `p`).
pub(crate) fn provides_energy_into<F: crate::derived::Facts>(g: &mut F, me: CardId, pe: &ProvidesEnergySpec, p: u8, source: SlotRef, map: &mut crate::effects::EnergyMap) -> R {
    if !g.g().st.slot(source.p as usize, source.s).cards.contains(me) {
        return Ok(());
    }
    let _ = p;
    if pe.probe && crate::derived::special_energy_off(g.g(), me) {
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
        map.push(EnergyEntry { card: me, provides });
    }
    Ok(())
}

/// `ProvidesEnergyBoost` of `me`: the entries it adds to the Energy map of the Pokémon in `source` (asked by `p`).
pub(crate) fn provides_energy_boost_into<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, b: &ProvidesEnergyBoostSpec, p: usize, source: SlotRef, map: &mut crate::effects::EnergyMap) -> R {
    let Some(at) = locate(g.g(), me, origin) else { return Ok(()) };
    if at.owner != p || blocked(g, me, origin, at, Some(source)) {
        return Ok(());
    }
    let cards: SVec<CardId, 60> = {
        let mut v = SVec::new();
        for c in g.g().st.slot(source.p as usize, source.s).cards.iter() {
            v.push(c);
        }
        v
    };
    for &c in cards.iter() {
        if !pred(g.g(), c, &b.energy) {
            continue;
        }
        if map.iter().any(|m| m.card == c) {
            continue;
        }
        let mut provides = SVec::new();
        for t in b.provides {
            provides.push(*t);
        }
        map.push(EnergyEntry { card: c, provides });
    }
    Ok(())
}

/// May the Energy `me` be attached to `target` (its own `AttachGuard`)?
pub(crate) fn attach_guard_allows<F: crate::derived::Facts>(g: &mut F, me: CardId, target: SlotRef, a: &AttachGuardSpec) -> R<bool> {
    slot_pred_m(g, me, target, &a.allow)
}

/// Is attaching the Energy card `card` to `target` stopped by its own `AttachGuard` (the Attach routine refuses
/// it with `CANNOT_PLAY_THIS_CARD`, `engine::attach::check_attach_with`; legality asks the same)?
pub fn attach_guard_refuses<F: crate::derived::Facts>(g: &mut F, card: CardId, target: SlotRef) -> R<bool> {
    let passives: &'static [Passive] = crate::cards::spec_for(g.g().st.cards[card as usize].def).map_or(&[], |s| s.passives);
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
            if !g.st.slot(p, target.s).cards.contains(card) || g.st.slot_pokemon(p, target.s).is_none() || crate::derived::special_energy_off(g, card) {
                continue;
            }
            if !slot_pred_m(g, card, target, &a.allow)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// The Energy's own `AttachGuard` at the table-state check of the state check: attached to a Pokémon it can't be on, it
/// is discarded ("discard this card": a LeavePlay of the attached card by its own rule).
pub(crate) fn attach_guard_now(g: &mut Game, me: CardId, a: &AttachGuardSpec) -> R {
    for p in 0..2usize {
        for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            let t = SlotRef::new(p, s);
            if !g.st.slot(p, s).cards.contains(me) || crate::derived::special_energy_off(g, me) {
                continue;
            }
            if g.st.slot_pokemon(p, s).is_some() && !slot_pred_m(g, me, t, &a.allow)? {
                let cause = crate::cause::Cause::of_origin(RuleSource::Energy, me, p as u8);
                crate::engine::knockout::leave_play_cards(g, t, &[me], super::event::RulesZone::Discard, cause, None)?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Costs

/// `AttackCost` of `me`: its change to the cost being computed (`check::AttackCostAcc`; the derived layer's
/// `attack_cost`).
pub(crate) fn attack_cost_into<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, c: &AttackCostSpec, acc: &mut crate::engine::check::CostAcc) -> R {
    let (p, attack) = (acc.p as usize, acc.attack.expect("an attack's cost"));
    if let Some(i) = c.attack {
        if attack != my_attack(g.g(), me, i) {
            return Ok(());
        }
    }
    let Some(at) = locate(g.g(), me, origin) else { return Ok(()) };
    if c.side == Side::Owner && at.owner != p || c.side == Side::Opponent && at.owner == p {
        return Ok(());
    }
    let active = SlotRef::new(p, g.g().st.players[p].active);
    if !slot_pred_m(g, me, active, &c.subject)? || blocked(g, me, origin, at, Some(active)) || !ability_effect_reaches(g, me, origin, at.owner, active, false)? {
        return Ok(());
    }
    let n = {
        let f = run::Frame::passive(g.g(), me, at.owner, origin);
        if !cond(g.g(), me, &f, &c.guard) {
            return Ok(());
        }
        match &c.change {
            CostChange::Reduce(n) => num(g.g(), me, &f, n),
            _ => 0,
        }
    };
    match &c.change {
        CostChange::AnyOne => acc.any_less = true,
        // D-11 / D-12: every "more" and "less" stacks and is calculated together, after all of them (`check::finish_cost`).
        CostChange::Reduce(_) => acc.less = acc.less.saturating_add(n.max(0) as u8),
        CostChange::Add(k) => acc.more = acc.more.saturating_add(*k as u8),
        CostChange::Free => acc.has_no = true,
        // D-13: "ignore all" takes priority over every "more" and "less".
        CostChange::IgnoreColorless => acc.ignore_colorless = true,
        // A cost that is set is not increased or decreased (R7F-11; D-13).
        CostChange::SetCost(set) => {
            let mut v: crate::effects::Cost = SVec::new();
            for t in set.iter() {
                v.push(*t);
            }
            acc.set_cost = Some(v);
        }
    }
    Ok(())
}

/// `RetreatCost` of `me`: its change to the Retreat Cost being computed (`check::RetreatCostAcc`; the derived layer's
/// `retreat_cost`).
pub(crate) fn retreat_cost_into<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, c: &RetreatCostSpec, acc: &mut crate::engine::check::CostAcc) -> R {
    let p = acc.p as usize;
    let Some(at) = locate(g.g(), me, origin) else { return Ok(()) };
    if (c.side == Side::Owner && at.owner != p) || (c.side == Side::Opponent && at.owner == p) {
        return Ok(());
    }
    let mut matched = None;
    let sides: &[usize] = if c.which == RetreatWhich::Mine { &[p] } else { &[p, 1 - p] };
    for q in sides {
        let active = SlotRef::new(*q, g.g().st.players[*q].active);
        if slot_pred_m(g, me, active, &c.subject)? {
            matched = Some(active);
            break;
        }
    }
    let Some(slot) = matched else { return Ok(()) };
    if blocked(g, me, origin, at, Some(slot)) || !ability_effect_reaches(g, me, origin, at.owner, slot, false)? {
        return Ok(());
    }
    let n = {
        let f = run::Frame::passive(g.g(), me, at.owner, origin);
        if !cond(g.g(), me, &f, &c.guard) {
            return Ok(());
        }
        match &c.change {
            CostChange::Reduce(n) => num(g.g(), me, &f, n),
            _ => 0,
        }
    };
    match &c.change {
        // D-11 / D-12: every "more" and "less" stacks and is calculated together (`check::finish_cost`).
        CostChange::Reduce(_) => acc.less = acc.less.saturating_add(n.max(0) as u8),
        CostChange::Add(k) => acc.more = acc.more.saturating_add(*k as u8),
        // D-13: "has no Retreat Cost" / "ignore all" take priority over every "more" and "less".
        CostChange::Free => acc.has_no = true,
        CostChange::IgnoreColorless => acc.ignore_colorless = true,
        CostChange::SetCost(_) | CostChange::AnyOne => {}
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Special Conditions

/// "Recovers from all Special Conditions": at every table-state check of the state check, each Pokémon matching the
/// subject recovers (one RemoveCondition per condition, by the card's effect).
pub(crate) fn recover_now(g: &mut Game, me: CardId, origin: RuleSource, c: &RecoverSpec) -> R {
    if g.st.any_special_condition() {
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
fn is_locked<F: crate::derived::Facts>(g: &mut F, me: CardId, card: CardId, l: &AbilityLockSpec) -> R<bool> {
    let owner = locker_owner(g.g(), me, l.locker);
    let Some(owner) = owner else { return Ok(false) };
    let slot = match g.g().st.locate(card) {
        None => return Ok(pred(g.g(), card, &l.missing)),
        Some(ListRef::Slot(q, s)) => SlotRef::new(q as usize, s),
        Some(_) => return Ok(false),
    };
    // "Has no Abilities" from an opponent's Ability: a continuous effect done to the Pokémon (batch 8 C11).
    let origin = lock_passives(g.g(), me).find(|ps| matches!(&ps.modifier, Modifier::AbilityLock(x) if std::ptr::eq(x, l))).map_or(RuleSource::Ability, |ps| ps.origin);
    if !lock_reaches(g, me, origin, card) {
        return Ok(false);
    }
    if !pred(g.g(), card, &l.card) {
        return Ok(false);
    }
    // A lock source's Ability is turned off only by a lock that wins over it.
    if lock_passives(g.g(), card).next().is_some() && !lock_beats(g, me, card)? {
        return Ok(false);
    }
    match l.probe {
        LockerProbe::Generic => {
            if !slot_pred_m(g, me, slot, &l.slot)? || g.has_no_ability(owner, me, None) {
                return Ok(false);
            }
        }
        LockerProbe::OwnPower(i) => {
            if !slot_pred_m(g, me, slot, &l.slot)? {
                return Ok(false);
            }
            let power = crate::effects::PowerRef { card: me, index: i };
            // CAN_APPLY_LOCKER_ABILITY: a real use of the locker's power must not be blocked.
            if power_blocked(g, owner, power, me, false) {
                return Ok(false);
            }
        }
        LockerProbe::StadiumOnSlot => {
            if crate::derived::stadium_off(g.g()) {
                return Ok(false);
            }
            match slot_pred_m(g, me, slot, &l.slot) {
                Ok(v) => {
                    if !v {
                        return Ok(false);
                    }
                }
                // The type check failed: the printed data decides.
                Err(_) => return Ok(pred(g.g(), card, &l.missing)),
            }
        }
    }
    Ok(true)
}

fn ability_lock(g: &mut Game, me: CardId, e: EffId, l: &AbilityLockSpec) -> R {
    match *g.e(e) {
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
pub(crate) const fn prevention_recovers(p: &PreventSpec) -> bool {
    !p.from.is_never() && p.from.effect_kinds().has(crate::effects::k::GAIN_CONDITION) && !p.from.reads_cause()
}

/// The recovery of [`prevention_recovers`], at the table-state check (where `Modifier::Recover` runs): each Pokémon
/// the declaration protects from a condition it has recovers from it (RemoveCondition by the protecting card).
pub(crate) fn recover_protected(g: &mut Game, me: CardId, origin: RuleSource, p: &PreventSpec) -> R {
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
    // A declaration over events is read by the event's routine (`event_prevented`), not here.
    if matches!(p.what, PreventWhat::None) {
        return Ok(());
    }
    if locate(g, me, origin).is_none() {
        return Ok(());
    }
    match (p.what, *g.e(e)) {
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
/// its own Pokémon: `Lasting::PreventAttackEffects`), stored on the Pokémon (the lasting table, `engine::lasting`).
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
/// Pokémon)" an attack leaves on its Pokémon (`DamageSource::spec`), stored with it (the lasting table, `engine::lasting`). The
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

/// Every lasting `Prevent` an attack can leave on a Pokémon (a `LastingWhat::Prevent` stores an index into it).
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

/// `PrizeAdjustOnce` (Legacy Energy): the adjustment, once per game for the Knocked Out Pokémon's player.
pub(crate) fn prize_adjust_once(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &PrizeAdjustSpec) -> R {
    let Effect::KnockOut { p, .. } = *g.e(e) else { return Ok(()) };
    let p = p as usize;
    if g.st.players[p].legacy_energy_used {
        return Ok(());
    }
    let before = match *g.e(e) {
        Effect::KnockOut { prize_count, .. } => prize_count,
        _ => 0,
    };
    prize_adjust(g, me, e, origin, d)?;
    if matches!(*g.e(e), Effect::KnockOut { prize_count, .. } if prize_count != before) {
        g.st.players[p].legacy_energy_used = true;
        g.st.bump();
    }
    Ok(())
}

/// `PrizeAdjust` of `me` over the KnockOut `e`: its change to the Prize count.
pub(crate) fn prize_adjust(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &PrizeAdjustSpec) -> R {
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
        // The attack being done (a copied one too: its effect applies, id2094), by this Pokémon (the copycat for a copy).
        match g.last_attack {
            Some(la) if la.p as usize == attacker && la.pokemon == Some(me) && crate::engine::attack::attack_def(g, la.attack).name == name => {}
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
        // A write inside the KnockOut's handlers (the derived facts are stale).
        g.st.bump();
    }
    if let Effect::KnockOut { prize_count, .. } = g.e_mut(e) {
        *prize_count += d.delta;
    }
    Ok(())
}

/// `GrantAttacks` of `me` (Relicanth's Memory Dive): the attacks it adds to player `p`'s attack list.
pub(crate) fn grant_attacks_into<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, p: usize, acc: &mut crate::engine::turn::CheckedAttacks) -> R {
    let Some(at) = locate(g.g(), me, origin) else { return Ok(()) };
    if at.owner != p || blocked(g, me, origin, at, None) {
        return Ok(());
    }
    let mut add: SVec<AttackRef, 32> = SVec::new();
    let active = g.g().st.players[p].active;
    if let Some(top) = g.g().st.slot_pokemon(p, active) {
        if g.g().st.cdef(top).stage != Stage::Basic as u8 {
            for c in g.g().st.slot(p, active).cards.iter() {
                let d = g.g().st.cdef(c);
                if d.is_pokemon() && c != top {
                    for i in 0..d.attacks.len() {
                        add.push(AttackRef { card: c, index: i as u8 });
                    }
                }
            }
        }
    }
    for a in add.iter() {
        acc.0.push(*a);
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
pub fn event_locked<F: crate::derived::Facts>(g: &mut F, v: &super::event::EventView) -> R<Option<&'static str>> {
    // A Pokémon event or an Attach without a card is one whose card isn't chosen yet (legality asks it only with
    // one). A CoinFlip has no card (a coin isn't a card): `EventPred::Card(..)` is false on it (`v.card` stays None;
    // test `coin_flip_has_no_card`); the probe below only orders the lock cards, whatever its card.
    if v.card.is_none() && v.kind != super::event::EventKind::CoinFlip {
        return Ok(None);
    }
    let p = v.actor() as usize;
    if !may_lock_event(g.g(), p, v.kind) {
        return Ok(None);
    }
    if let (Some(marker), Some(kind)) = (lock_marker(v.kind), v.kind.effect_kind()) {
        if g.g().kinds_present.has(marker) {
            // The lock sources of the event's kind, in the propagation order (`decl::kind_listeners`: no dispatch index,
            // a plain read).
            let order = g.decl_kind_listeners(kind);
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
        E::Discard | E::Draw | E::PutIntoHand | E::PutIntoDeck | E::LeavePlay => Some(crate::effects::k::DECLARES_CARD_LOCK),
        E::PlayTrainer => Some(crate::effects::k::DECLARES_PLAY_LOCK),
        E::UseAttack | E::UseAbility | E::UseStadium => Some(crate::effects::k::DECLARES_USE_LOCK),
        E::Damage
        | E::ApplyEffect
        | E::ContinuousEffect
        | E::KnockOut
        | E::TakePrizes
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
        | E::Retreat => None,
    }
}

/// Can a lock forbid an event of `kind` of player `p` at all (a plain read: some card of the game declares a
/// lock over events of its family, or an attack left a lock over events on `p`)? When it can't,
/// [`event_locked`] answers `None` without a walk, so legality asks it before making its scratch game.
#[inline]
pub fn may_lock_event(g: &Game, p: usize, kind: super::event::EventKind) -> bool {
    lock_marker(kind).map_or(false, |m| g.kinds_present.has(m))
        || crate::engine::lasting::any_lock_of(&g.st, p)
}

/// Does a lock an attack left on player `p` or on one of their Pokémon read the Energy the attacking Pokémon provides
/// (`EventPred::EnergyAtMost`: Walrein's Frigid Fangs, on each Pokémon since batch 8 C12c)? The UseAttack's checks make
/// that checked read only then.
pub fn lasting_reads_energy(g: &Game, p: usize) -> bool {
    g.st.lasting.iter().any(|l| match (l.on, l.what) {
        (crate::state::LastingOn::Player(q), crate::state::LastingWhat::PlayerLock { decl, .. }) => q as usize == p && decl.forbids.reads_energy(),
        (crate::state::LastingOn::Pokemon(q, _), crate::state::LastingWhat::Lock { decl, .. }) => q as usize == p && decl.forbids.reads_energy(),
        _ => false,
    })
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
        E::ContinuousEffect => Some(crate::effects::k::DECLARES_CONTINUOUS_PREVENT),
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
/// A re-flip declaration (`Modifier::Reflip`): the group of flips it covers (`on`, over the CoinFlip event of the group:
/// its purpose and cause) and the marker that makes it once per turn (set on its player when used, cleared at the end
/// of their turn by the card's own trigger). The player asked is the declaring card's owner, who must be the one
/// flipping.
pub struct ReflipSpec {
    pub on: super::event::EventPred,
    pub once: crate::markers::MarkerName,
}

/// The re-flip declaration in play that covers the group of flips `v` (a CoinFlip view: purpose, cause, the player
/// flipping): its card is in place for its origin and not blocked there, its owner is the one flipping, its marker isn't
/// set, and `on` matches (the view's owner is the player flipping). Returns the marker. A game without one answers `None`
/// with a mask test.
pub fn reflip_offered(g: &mut Game, v: &super::event::EventView) -> R<Option<crate::markers::MarkerName>> {
    if !g.kinds_present.has(crate::effects::k::DECLARES_REFLIP) {
        return Ok(None);
    }
    let order = g.decl_kind_listeners(crate::effects::k::COIN_FLIP);
    for c in order.iter().copied() {
        let passives: &'static [Passive] = crate::cards::spec_for(g.st.cards[c as usize].def).map_or(&[], |s| s.passives);
        for ps in passives {
            let Modifier::Reflip(r) = &ps.modifier else { continue };
            let Some(at) = locate(g, c, ps.origin) else { continue };
            if at.owner != v.owner as usize || g.st.players[at.owner].marker.has(r.once) {
                continue;
            }
            if blocked(g, c, ps.origin, at, at.held) || !r.on.eval(g, c, v)? {
                continue;
            }
            return Ok(Some(r.once));
        }
    }
    Ok(None)
}

pub fn event_prevented(g: &mut Game, v: &super::event::EventView) -> R<bool> {
    let (Some(marker), Some(kind)) = (prevent_marker(v.kind), v.kind.effect_kind()) else { return Ok(false) };
    if !g.kinds_present.has(marker) {
        return Ok(false);
    }
    let order = g.decl_kind_listeners(kind);
    for c in order.iter().copied() {
        if prevented_by(g, c, v)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The `Prevent` declarations an attack left on the event's Pokémon ("during your opponent's next turn, prevent all
/// damage done to / effects of attacks done to this Pokémon": the lasting table's preventions, in force once no longer
/// pending): one matches the event. Each is evaluated for the card whose attack left it. A plain read of the slot when
/// it holds none.
pub fn lasting_prevented(g: &mut Game, v: &super::event::EventView) -> R<bool> {
    let Some(t) = v.slot else { return Ok(false) };
    for l in crate::engine::lasting::prevents(&g.st, t.p as usize, t.s).iter() {
        let crate::state::LastingWhat::Prevent { spec, pending } = l.what else { continue };
        let spec = LASTING_PREVENTS[spec as usize];
        if pending || spec.coin || !ranges(&spec.from, v.kind) {
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
            Some(s) => slot_pred_m(g, me, s, &p.protects)? && ability_effect_reaches(g, me, ps.origin, at.owner, s, false)?,
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
    let order = g.decl_kind_listeners(kind);
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
                Some(s) => slot_pred_m(g, me, s, &p.protects)? && ability_effect_reaches(g, me, ps.origin, at.owner, s, false)?,
                None => matches!(p.protects, SlotPred::Any),
            };
            if !protected || blocked(g, me, ps.origin, at, v.slot) {
                continue;
            }
            let cause = crate::cause::Cause::of_origin(ps.origin, me, at.owner as u8);
            if crate::engine::condition::flip_coin(g, at.owner, crate::game::CoinCb::None, cause)? {
                heads = true;
            }
        }
    }
    Ok(heads)
}

/// [`event_locked`] for one in-play lock source `me`.
pub(crate) fn event_locked_by<F: crate::derived::Facts>(g: &mut F, me: CardId, v: &super::event::EventView) -> R<Option<&'static str>> {
    let passives: &'static [Passive] = crate::cards::spec_for(g.g().st.cards[me as usize].def).map_or(&[], |s| s.passives);
    for ps in passives {
        if let Modifier::BlockUse(b) = &ps.modifier {
            if let Some(code) = event_lock_blocks(g, me, ps.origin, b, v)? {
                return Ok(Some(code));
            }
        }
    }
    Ok(None)
}

/// [`event_locked`] for the locks an attack left: on the event's actor (the locked player), and on the Pokémon the event
/// is about (its spot, a retreat's leaving Pokémon: "this Pokémon can't attack / retreat during your next turn", APR
/// C-15), evaluated for that Pokémon; a lock naming an attack forbids only that attack.
pub fn lasting_event_locked<F: crate::derived::Facts>(g: &mut F, v: &super::event::EventView) -> R<Option<&'static str>> {
    use crate::state::LastingWhat;
    let p = v.actor() as usize;
    for l in crate::engine::lasting::player_locks(&g.g().st, p).iter() {
        let LastingWhat::PlayerLock { decl, .. } = l.what else { continue };
        if decl.coin == CoinGate::No && !decl.forbids.is_never() && decl.forbids.eval(g, l.source, v)? {
            return Ok(Some(decl.error));
        }
    }
    let spots = [v.slot, v.from.filter(|f| Some(*f) != v.slot)];
    for s in spots.into_iter().flatten() {
        let locks = crate::engine::lasting::pokemon_locks(&g.g().st, s.p as usize, s.s);
        if locks.is_empty() {
            continue;
        }
        let Some(me) = g.g().st.slot_pokemon(s.p as usize, s.s) else { continue };
        for l in locks.iter() {
            let LastingWhat::Lock { decl, attack, .. } = l.what else { continue };
            let named = match attack {
                None => true,
                Some(a) => v.attack.map_or(false, |x| crate::engine::attack::attack_def(g.g(), x).name == crate::engine::lasting::lock_attack_name(&g.g().st, a)),
            };
            if named && decl.coin == CoinGate::No && decl.forbids.eval(g, me, v)? {
                return Ok(Some(decl.error));
            }
        }
    }
    Ok(None)
}

/// A coin-gated lock an attack left on the event's actor that the event matches (Seismitoad's Quaking Fist: "whenever
/// they try to use a Trainer card from their hand, they flip a coin"): the cause of the coin (the lock's source's
/// attack, by its owner), or `None` when no such lock applies.
pub fn coin_gate(g: &mut Game, v: &super::event::EventView) -> R<Option<crate::cause::Cause>> {
    let p = v.actor() as usize;
    for l in crate::engine::lasting::player_locks(&g.st, p).iter() {
        let crate::state::LastingWhat::PlayerLock { decl, .. } = l.what else { continue };
        if decl.coin != CoinGate::No && decl.forbids.eval(g, l.source, v)? {
            return Ok(Some(crate::cause::Cause::new(crate::cause::CauseKind::Attack, Some(l.source), g.st.owner(l.source) as u8)));
        }
    }
    Ok(None)
}

/// The cause of the coin-gated lock on player `p` (its tails' discard), if one is there.
pub fn coin_gate_cause(g: &Game, p: usize) -> Option<crate::cause::Cause> {
    crate::engine::lasting::player_locks(&g.st, p)
        .iter()
        .find(|l| matches!(l.what, crate::state::LastingWhat::PlayerLock { decl, .. } if decl.coin != CoinGate::No))
        .map(|l| crate::cause::Cause::new(crate::cause::CauseKind::Attack, Some(l.source), g.st.owner(l.source) as u8))
}

/// Does the in-play lock `b` of `me` forbid the event (`LockDecl::forbids`)?
pub(crate) fn event_lock_blocks<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, b: &BlockUseSpec, v: &super::event::EventView) -> R<Option<&'static str>> {
    if b.lock.forbids.is_never() {
        return Ok(None);
    }
    let Some(at) = locate(g.g(), me, origin) else { return Ok(None) };
    // The locked player is the actor (`EventView::actor`).
    let p = v.actor() as usize;
    let binds = match b.binds {
        Binds::Opponent => p == 1 - at.owner,
        Binds::Owner => p == at.owner,
        Binds::Both => true,
    };
    if !binds || !crate::engine::enter::while_ok(g.g(), me, at, b.while_, v.card) {
        return Ok(None);
    }
    if !b.lock.forbids.eval(g, me, v)? {
        return Ok(None);
    }
    if b.ability && !source_ability_on(g, at.owner, me) {
        return Ok(None);
    }
    // A lock of an opponent's Ability on an event done to a Pokémon: a continuous effect done to that Pokémon.
    if let Some(s) = v.slot {
        if !ability_effect_reaches(g, me, origin, at.owner, s, false)? {
            return Ok(None);
        }
    }
    Ok(Some(b.lock.error))
}

/// Does the lock source `me` have its Ability (`derived::has_no_ability`, the lock pass)?
fn source_ability_on<F: crate::derived::Facts>(g: &mut F, owner: usize, me: CardId) -> bool {
    !g.has_no_ability(owner, me, None)
}

/// The lock pass for one use of a power (what the `Power` effect's handlers decided; a lock probe when `probe`, a real
/// use otherwise): is it blocked? A copied attack's source code, run for the copycat, can't use the copycat's own
/// Abilities; otherwise each Ability lock of the game (`Game::lock_cards`, any zone: each checks where its source is) is
/// asked, undelegated as the card handlers were. A blocked answer of one lock, or an error of its reads, blocks.
pub(crate) fn power_blocked<F: crate::derived::Facts>(g: &mut F, p: usize, power: crate::effects::PowerRef, card: CardId, probe: bool) -> bool {
    if matches!(g.g().deleg, Some(d) if d.attacks && d.copycat == card) {
        return true;
    }
    if !probe && crate::game::verify_cache() {
        // A lock's "a real use of its own power" probe: no program would run (none of the pool's lock sources has one).
        assert!(!crate::spec::run::power_has_program(g.g(), power), "a lock probe of power {power:?}, which has a program");
    }
    let lockers = g.g().lock_cards;
    g.undelegated(|g| {
    let mut blocked = false;
    'cards: for (w, bits) in lockers.iter().enumerate() {
        let mut b = *bits;
        while b != 0 {
            let c = (w * 64 + b.trailing_zeros() as usize) as CardId;
            b &= b - 1;
            for ps in lock_passives(g.g(), c) {
                let hit = match &ps.modifier {
                    Modifier::ActiveLock(l) => active_lock_subject(g.g(), *l, power, probe) && !matches!(active_lock_applies(g, c, *l, p, card, true), Ok(false)),
                    Modifier::AbilityLock(l) => power_subject(g.g(), &l.powers, power) && !matches!(is_locked(g, c, card, l), Ok(false)),
                    _ => false,
                } && lock_reaches(g, c, ps.origin, card);
                if hit {
                    blocked = true;
                    break 'cards;
                }
            }
        }
    }
    blocked
    })
}

/// The powers player `p`'s Pokémon card `card` has now: its printed ones, less those each Ability lock of the game takes
/// away, in the propagation order of the powers a Pokémon has (`game::PROP_POWERS_NOW`).
pub(crate) fn powers_now<F: crate::derived::Facts>(g: &mut F, p: usize, card: CardId) -> R<SVec<crate::effects::PowerRef, 8>> {
    let mut powers: SVec<crate::effects::PowerRef, 8> = SVec::new();
    for i in 0..g.g().st.cdef(card).powers.len() {
        powers.push(crate::effects::PowerRef { card, index: i as u8 });
    }
    let lc = g.g().lock_cards;
    let set = (lc[0] as u128) | (lc[1] as u128) << 64;
    debug_assert!(lc[2] == 0 && lc[3] == 0);
    if set == 0 {
        return Ok(powers);
    }
    let order = g.decl_ordered(set, crate::game::PROP_POWERS_NOW);
    g.undelegated(|g| -> R<SVec<crate::effects::PowerRef, 8>> {
        for c in order.iter().copied() {
            for ps in lock_passives(g.g(), c) {
                match &ps.modifier {
                    Modifier::ActiveLock(l) => {
                        if active_lock_applies(g, c, *l, p, card, false)? && lock_reaches(g, c, ps.origin, card) {
                            let mut out = SVec::new();
                            for pw in powers.iter() {
                                if !active_lock_subject(g.g(), *l, *pw, false) {
                                    out.push(*pw);
                                }
                            }
                            powers = out;
                        }
                    }
                    Modifier::AbilityLock(l) => {
                        if is_locked(g, c, card, l)? && lock_reaches(g, c, ps.origin, card) {
                            let mut out = SVec::new();
                            for pw in powers.iter() {
                                if !power_subject(g.g(), &l.powers, *pw) {
                                    out.push(*pw);
                                }
                            }
                            powers = out;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(powers)
    })
}

/// Does the continuous effect of `source`'s Ability (its owner `owner`) reach the Pokémon in `target` (events batch 8 C11,
/// user decision 1; JP FAQ Banette vs Intimidating Fang, Sinistcha vs Excellent Pheromone; id2422, id2426)? An effect of a
/// Stadium, a Tool or any other origin, and an effect on its owner's own Pokémon, always reach. Otherwise the view of the
/// continuous effect done to the Pokémon (`EventKind::ContinuousEffect`, caused by the Ability) is asked of the `Prevent`
/// declarations ranging over it (Hide 'n' Sneak: "prevent all effects of your opponent's Pokémon's Abilities done to this
/// Pokémon"). `in_lock_pass`: asked by the lock pass, where a protecting declaration counts in place whatever locks its own
/// Ability (a lock that turns it off turns the protected Pokémon's Ability off itself: the same answer, without the cycle;
/// RULES.md precedence 1).
pub(crate) fn ability_effect_reaches<F: crate::derived::Facts>(g: &mut F, source: CardId, origin: RuleSource, owner: usize, target: SlotRef, in_lock_pass: bool) -> R<bool> {
    if origin != RuleSource::Ability || target.p as usize == owner || !g.g().kinds_present.has(crate::effects::k::DECLARES_CONTINUOUS_PREVENT) {
        return Ok(true);
    }
    let Some(card) = g.g().st.slot_pokemon(target.p as usize, target.s) else { return Ok(true) };
    let cause = crate::cause::Cause::new(crate::cause::CauseKind::Ability, Some(source), owner as u8);
    let v = super::event::EventView { card: Some(card), slot: Some(target), ..super::event::EventView::new(super::event::EventKind::ContinuousEffect, cause, target.p, super::event::whose_turn(g.g())) };
    let order = g.decl_kind_listeners(crate::effects::k::CONTINUOUS_EFFECT);
    for c in order.iter().copied() {
        let passives: &'static [Passive] = crate::cards::spec_for(g.g().st.cards[c as usize].def).map_or(&[], |s| s.passives);
        for ps in passives {
            let Modifier::Prevent(p) = &ps.modifier else { continue };
            if p.from.is_never() || p.coin || !ranges(&p.from, v.kind) || !p.from.eval(g, c, &v)? {
                continue;
            }
            let Some(at) = locate(g.g(), c, ps.origin) else { continue };
            if slot_pred_m(g, c, target, &p.protects)? && (in_lock_pass || !blocked(g, c, ps.origin, at, Some(target))) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// [`ability_effect_reaches`] for an Ability lock of `source` on `card` ("has no Abilities"), in the lock pass.
fn lock_reaches<F: crate::derived::Facts>(g: &mut F, source: CardId, origin: RuleSource, card: CardId) -> bool {
    let Some(ListRef::Slot(q, s)) = g.g().st.locate(card) else { return true };
    let owner = g.g().st.locate(source).and_then(|l| l.owner()).unwrap_or_else(|| g.g().st.owner(source));
    !matches!(ability_effect_reaches(g, source, origin, owner, SlotRef::new(q as usize, s), true), Ok(false))
}

/// Does the effect of `card`'s Ability `power` reach the Pokémon in `target` (what the `EffectOfAbility` probe's
/// handlers decided)? Iron Thorns ex's Initialization can't change the effect of its own Ability on a Future Pokémon;
/// nothing else in the game changes one.
pub(crate) fn effect_of_ability_reaches(g: &Game, card: CardId, power: crate::effects::PowerRef, target: SlotRef) -> bool {
    let own_initialization = power == (crate::effects::PowerRef { card, index: 0 }) && lock_passives(g, card).any(|ps| matches!(ps.modifier, Modifier::ActiveLock(ActiveLock::Initialization)));
    if own_initialization {
        if let Some(c) = g.st.slot_pokemon(target.p as usize, target.s) {
            if g.st.cdef(c).has_tag(tag::FUTURE) {
                return false;
            }
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Survive on 10 (S3 agent 3)

/// `SurviveOnTen` of `me`: a survive-on-10 replacement armed for the damage being calculated (step 5).
pub(crate) fn survive_on_ten_into(g: &mut Game, me: CardId, origin: RuleSource, spec: &SurviveOnTenSpec, acc: &mut DamageAcc) -> R {
    let (t, damage) = (acc.b.target, acc.damage);
    if spec.kind == SurviveKind::ToolIfFullHp {
        return survive_on_ten_tool(g, me, origin, acc);
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
            survive_on_ten_on_coin_flip(g, acc, owner, crate::cause::Cause::of_origin(origin, me, owner as u8))?
        }
        SurviveKind::IfFullHp => {
            if g.st.slot(owner, t.s).damage != 0 {
                return Ok(());
            }
            let hp = crate::derived::hp(g, owner, t.s)?;
            if damage >= hp {
                acc.survive = true;
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
    // "Put N more damage counters on your opponent's Burned / Poisoned Pokémon" is a continuous effect of the Ability
    // done to that Pokémon (id2112): a `Prevent` over the Ability's effects keeps it off (Hide 'n' Sneak vs Magmortar's
    // Magma Surge; batch 8 C12b, review 8b Must 3).
    if !ability_effect_reaches(g, me, origin, at.owner, victim, false)? {
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

fn next_turn_bonus(g: &mut Game, me: CardId, e: EffId, b: &'static NextTurnBonusSpec) -> R {
    let (attack, source) = match *g.e(e) {
        Effect::Attack { attack, source, .. } => (attack, source),
        _ => return Ok(()),
    };
    if g.st.slot_pokemon(source.p as usize, source.s) != Some(me) {
        return Ok(());
    }
    let full_name = g.st.cdef(me).full_name;
    let attack_name = g.st.cdef(attack.card).attacks[attack.idx()].name;
    let armed = match crate::engine::lasting::armed_bonus(&g.st, source.p as usize, source.s) {
        Some((a, for_card)) if g.st.cdef(for_card).full_name == full_name && (a.attack == "*" || a.attack == attack_name) => a.bonus,
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
    crate::engine::lasting::set_bonus_pending(&mut g.st, source.p as usize, source.s, b, me);
    g.st.bump();
    Ok(())
}

fn attack_flags(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, a: &AttackFlagsSpec) -> R {
    match &a.shred {
        Some(attacker) => shred_flags(g, me, e, origin, attacker),
        // The first-turn permission is a declaration the UseAttack's checks read (`grants_first_turn_attack`).
        None => Ok(()),
    }
}

/// Does `me` (an Ability on the attacking player's Active Pokémon) let `p`'s attacks be used on the first turn?
fn first_turn_grants<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, a: &AttackFlagsSpec, p: usize) -> bool {
    let active = g.g().st.players[p].active;
    if !a.first_turn || !g.g().st.slot(p, active).cards.contains(me) || g.g().st.turn != 1 {
        return false;
    }
    let at = Located { owner: p, held: None };
    !blocked(g, me, origin, at, None)
}

/// The first-turn attack permission ("if you go first, this Pokémon can attack during your first turn": Meloetta ex's Live
/// Debut, `AttackFlags { first_turn }`): does an Ability on player `p`'s Active Pokémon let its attacks be used on the
/// first turn now? The UseAttack's checks read it (execution and legality alike).
pub fn grants_first_turn_attack<F: crate::derived::Facts>(g: &mut F, p: usize) -> bool {
    if g.g().st.turn != 1 || g.g().decls.sets[crate::decl::fam::FIRST_TURN] == 0 {
        return false;
    }
    let active = g.g().st.players[p].active;
    let mut cards: SVec<CardId, 8> = SVec::new();
    for c in g.g().st.slot(p, active).cards.iter() {
        cards.push(c);
    }
    for me in cards.iter().copied() {
        let passives: &'static [Passive] = crate::cards::spec_for(g.g().st.cards[me as usize].def).map_or(&[], |s| s.passives);
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

/// `BenchAttacks` of `me` (Mew ex's Memory Helix): the Benched Pokémon's attacks it adds to player `p`'s attack list, as
/// copied ones.
pub(crate) fn bench_attacks_into<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, p: usize, acc: &mut crate::engine::turn::CheckedAttacks) -> R {
    if g.g().st.active_pokemon(p) != Some(me) || blocked(g, me, origin, Located { owner: p, held: None }, None) {
        return Ok(());
    }
    let mut add: SVec<AttackRef, 32> = SVec::new();
    let bench: Vec<crate::state::SlotId> = g.g().st.players[p].bench.iter().copied().collect();
    for b in bench {
        if let Some(c) = g.g().st.slot_pokemon(p, b) {
            for i in 0..g.g().st.cdef(c).attacks.len() {
                add.push(AttackRef { card: c, index: i as u8 });
            }
        }
    }
    for a in add.iter() {
        acc.0.push(*a);
        acc.1.push(*a);
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
fn lock_covers<F: crate::derived::Facts>(g: &mut F, a: CardId, pa: &Passive, s: CardId) -> R<bool> {
    let Some(owner) = lock_position(g.g(), a, pa) else { return Ok(false) };
    if !lock_reaches(g, a, pa.origin, s) {
        return Ok(false);
    }
    let Some(power) = first_ability(g.g(), s) else { return Ok(false) };
    let slot = match g.g().st.locate(s) {
        Some(ListRef::Slot(q, sl)) => SlotRef::new(q as usize, sl),
        _ => return Ok(false),
    };
    match &pa.modifier {
        Modifier::ActiveLock(l) => {
            if !active_lock_subject(g.g(), *l, power, false) {
                return Ok(false);
            }
            Ok(match l {
                ActiveLock::MidnightFluttering => slot == SlotRef::new(1 - owner, g.g().st.players[1 - owner].active),
                ActiveLock::Initialization => {
                    let d = g.g().st.cdef(s);
                    !d.has_tag(tag::FUTURE) && d.has_rule_box()
                }
            })
        }
        Modifier::AbilityLock(l) => {
            if !power_subject(g.g(), &l.powers, power) || !pred(g.g(), s, &l.card) {
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
fn lock_beats<F: crate::derived::Facts>(g: &mut F, a: CardId, s: CardId) -> R<bool> {
    let mut mutual = false;
    for ps in lock_passives(g.g(), s) {
        if lock_covers(g, s, ps, a)? {
            mutual = true;
            break;
        }
    }
    Ok(!mutual || lock_earlier(g.g(), a, s))
}

/// Does the source of lock `p` have its Ability (it is in place, `owner`'s)? Probed as the lock itself does.
/// A Stadium has none to turn off.
fn lock_has_ability<F: crate::derived::Facts>(g: &mut F, c: CardId, p: &Passive, owner: usize) -> bool {
    if p.origin != RuleSource::Ability {
        return true;
    }
    let real = |g: &mut F, index: u8| !power_blocked(g, owner, crate::effects::PowerRef { card: c, index }, c, false);
    match &p.modifier {
        Modifier::ActiveLock(_) => real(g, 0),
        Modifier::AbilityLock(l) => match l.probe {
            LockerProbe::Generic => !g.has_no_ability(owner, c, None),
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
    if !crate::game::verify_cache() || !g.kinds_present.has(crate::effects::k::DECLARES_ABILITY_LOCK) {
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
    if !g.kinds_present.has(crate::effects::k::DECLARES_ABILITY_LOCK) {
        return;
    }
    // Batch 8 C6: a lock source holds by where the cards are (the layout), the turn (the turn player first when several
    // take hold together; the lasting "Stadium and Tool cards have no effect") and the stamps: with none of them changed
    // since the last sync, a sync changes nothing (`PTCG_VERIFY_CACHE=1` runs it and asserts so).
    let key = (g.st.layout_gen, g.st.turn, g.st.active_player, crate::derived::stadium_off(g));
    if g.st.lock_synced == Some(key) {
        if crate::game::verify_cache() {
            let before: Vec<u16> = (0..g.st.n_cards).map(|c| g.st.cards[c as usize].lock_stamp).collect();
            let counter = g.st.ability_lock_order_counter;
            lock_sync_now(g);
            let after: Vec<u16> = (0..g.st.n_cards).map(|c| g.st.cards[c as usize].lock_stamp).collect();
            assert!(before == after && counter == g.st.ability_lock_order_counter, "lock_sync with the layout and the turn unchanged changed the take-hold stamps");
        }
        return;
    }
    lock_sync_now(g);
    g.st.lock_synced = Some(key);
}

fn lock_sync_now(g: &mut Game) {
    // Only a card that declares an Ability lock can hold one or carry a stamp (`Game::lock_cards`).
    let lockers = g.lock_cards;
    let is_locker = |c: CardId| lockers[(c >> 6) as usize] & (1u64 << (c & 63)) != 0;
    if crate::game::verify_cache() {
        for c in 0..g.st.n_cards {
            assert_eq!(lock_passives(g, c).next().is_some(), is_locker(c), "Game::lock_cards is stale for card {c}");
        }
    }
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
                // A stamp is a fact the locks read (a write: `State::version`).
                g.st.bump();
                changed = true;
            } else if holds && stamp == 0 {
                taking.push(c);
            }
        }
        taking.sort_by_key(|c| lock_turn_order(g, *c));
        for c in taking {
            g.st.ability_lock_order_counter = g.st.ability_lock_order_counter.saturating_add(1);
            g.st.cards[c as usize].lock_stamp = g.st.ability_lock_order_counter;
            g.st.bump();
            changed = true;
        }
        if !changed {
            break;
        }
    }
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
fn active_lock_applies<F: crate::derived::Facts>(g: &mut F, me: CardId, l: ActiveLock, player: usize, card: CardId, power_effect: bool) -> R<bool> {
    let own = crate::effects::PowerRef { card: me, index: 0 };
    // "Has no Abilities" from an opponent's Ability: a continuous effect done to the Pokémon (batch 8 C11; Hide 'n' Sneak
    // over Midnight Fluttering, RULES.md precedence 1, id2426).
    if !lock_reaches(g, me, RuleSource::Ability, card) {
        return Ok(false);
    }
    match l {
        ActiveLock::MidnightFluttering => {
            // Only a Flutter Mane in its owner's Active Spot locks, and only the opponent's Active Pokémon; a card
            // anywhere else (a deck, a list of cards being looked at) is neither.
            let Some(owner) = g.g().st.locate(me).and_then(|l| l.owner()) else { return Ok(false) };
            if g.g().st.active_pokemon(owner) != Some(me) {
                return Ok(false);
            }
            let opponent = 1 - owner;
            if g.g().st.locate(card) != Some(ListRef::Slot(opponent as u8, g.g().st.players[opponent].active)) {
                return Ok(false);
            }
            // LOCKER_ABILITY_APPLIES: a lock source is turned off only by a lock that wins over it.
            if lock_passives(g.g(), card).next().is_some() && !lock_beats(g, me, card)? {
                return Ok(false);
            }
            Ok(!power_blocked(g, owner, own, me, false))
        }
        ActiveLock::Initialization => {
            if g.g().st.active_pokemon(player) != Some(me) && g.g().st.active_pokemon(1 - player) != Some(me) {
                return Ok(false);
            }
            // A card in the hand is judged by its printed data (docs/rulings/RULES.md): only Pokémon in play are locked.
            let slot = match g.g().st.locate(card) {
                Some(ListRef::Slot(q, s)) => SlotRef::new(q as usize, s),
                _ => return Ok(false),
            };
            let d = g.g().st.cdef(card);
            if d.has_tag(tag::FUTURE) || !d.has_rule_box() {
                return Ok(false);
            }
            let locker_owner = if g.g().st.active_pokemon(player) == Some(me) { player } else { 1 - player };
            // LOCKER_ABILITY_APPLIES
            if lock_passives(g.g(), card).next().is_some() && !lock_beats(g, me, card)? {
                return Ok(false);
            }
            if power_blocked(g, locker_owner, own, me, false) {
                return Ok(false);
            }
            if power_effect {
                // CAN_APPLY_LOCK_TO_TARGET
                return Ok(effect_of_ability_reaches(g.g(), me, own, slot));
            }
            Ok(true)
        }
    }
}

fn active_lock(g: &mut Game, me: CardId, e: EffId, l: ActiveLock) -> R {
    match *g.e(e) {
        Effect::Power { p, power, card, .. } => {
            if active_lock_subject(g, l, power, false) && active_lock_applies(g, me, l, p as usize, card, true)? {
                crate::bail!("BLOCKED_BY_ABILITY");
            }
        }
        _ => {}
    }
    Ok(())
}

/// A guard evaluated with checked reads (Energy provided, types as the game checks them).
fn guard_ok_m<F: crate::derived::Facts>(g: &mut F, me: CardId, origin: RuleSource, owner: usize, guard: &Cond) -> R<bool> {
    let f = run::Frame::passive(g.g(), me, owner, origin);
    cond_m(g, me, &f, guard)
}

/// A Benched Pokémon whose counters come from the opponent's Pokémon (Battle Cage).

/// The Bench size the Stadium's rule gives each player whose guard holds (the table-state check of the state check).
pub(crate) fn bench_size_into(g: &mut Game, me: CardId, origin: RuleSource, d: &BenchSizeSpec, sizes: &mut [u8; 2]) {
    if locate(g, me, origin).is_none() {
        return;
    }
    for (p, size) in sizes.iter_mut().enumerate() {
        if guard_ok(g, me, origin, p, &d.guard) {
            *size = d.size;
        }
    }
}

// ---------------------------------------------------------------------------
// Heavy Baton

// S3-4 appends: attack blocks, play blocks, survive on 10 HP, Shred

/// "If the Pokémon has full HP and would be Knocked Out by damage from an opponent's attack, it is not Knocked Out
/// and its remaining HP becomes 10 instead; then discard this card."
fn survive_on_ten_tool(g: &mut Game, me: CardId, origin: RuleSource, acc: &mut DamageAcc) -> R {
    let (t, damage, attacker) = (acc.b.target, acc.damage, acc.b.player as usize);
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
    acc.survive = true;
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
    let source = match *g.e(e) {
        Effect::Attack { source, .. } => source,
        _ => return Ok(()),
    };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if !slot_pred_m(g, me, source, attacker)? || blocked(g, me, origin, at, None) {
        return Ok(());
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
    //! `event_locked` and "has no Ability" (`derived::has_no_ability`) on the lock cards (the queries execution and
    //! legality share).
    use super::*;
    use serde_json::json;

    /// "Has no Ability" of `card` (the generic probe, for the player whose card it is where it is), on a copy of the game.
    fn ability_off(g: &Game, card: CardId) -> Option<bool> {
        let mut c = Box::new(*g);
        let owner = c.st.locate(card).and_then(|l| l.owner()).unwrap_or_else(|| c.st.owner(card));
        Some(crate::derived::has_no_ability(&mut c, owner, card, None))
    }

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
        use crate::engine::lasting::{add_player_lock, end_turn};
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
            add_player_lock(&mut g.st, me, decl, 1, 0);
            for c in [potion, research, tower] {
                let want = (c == stops).then_some("BLOCKED_BY_EFFECT");
                assert_eq!(play_locked(&mut g, me, c), want, "{} under a lock on {}", g.st.cdef(c).name, g.st.cdef(stops).name);
            }
            // The opponent of the locked player isn't stopped.
            assert_eq!(play_locked(&mut g, 1 - me, stops), None);
            end_turn(&mut g.st, 1 - me);
            assert!(play_locked(&mut g, me, stops).is_some(), "the other player's turn doesn't end it");
            end_turn(&mut g.st, me);
            assert_eq!(play_locked(&mut g, me, stops), None, "gone at the end of the locked player's turn");
        }
        // A used Supporter isn't played (id2226: "can't play Supporter cards" doesn't stop Look-Alike Show).
        add_player_lock(&mut g.st, me, &SUPPORTER, 1, 0);
        let attack = crate::cause::Cause::attack(me as u8, None, crate::state::AttackRef { card: 0, index: 0 });
        let used = crate::engine::play_trainer::play_view(&g, research, crate::spec::event::TrainerUse::Used, crate::spec::event::RulesZone::Hand, attack);
        assert_eq!(event_locked(&mut g, &used).unwrap(), None);
        // Seismitoad's coin-gated lock isn't a legality check (heads lets the play through): the lock query doesn't answer
        // it; the play flips (`coin_gate`).
        static QUAKING: LockDecl = LockDecl { error: "BLOCKED_BY_EFFECT", forbids: PLAY_TRAINER_FROM_HAND, coin: CoinGate::TailsDiscardsCard };
        let mut g2 = game(json!({"me": {"reset": true, "active": "Hoothoot PRE 77", "hand": ["Potion POR 83"]}, "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let potion = hand(&g2, me, "Potion POR 83");
        add_player_lock(&mut g2.st, me, &QUAKING, 1, 0);
        assert_eq!(play_locked(&mut g2, me, potion), None);
        let v = crate::engine::play_trainer::play_view(&g2, potion, crate::spec::event::TrainerUse::Played, crate::spec::event::RulesZone::Hand, crate::cause::Cause::rule(crate::cause::RuleWhich::Action, me as u8));
        assert!(coin_gate(&mut g2, &v).unwrap().is_some());
        // Bronzong's lock is over events: evolving from the hand (Rare Candy too), not putting onto the Bench.
        use crate::spec::event::{EventKind, EventPred, RulesZone};
        static EVOLVE: LockDecl = LockDecl::on(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::Source(RulesZone::Hand)]), "BLOCKED_BY_EFFECT");
        let noctowl = hand(&g, me, "Noctowl PRE 78");
        assert_eq!(event_lock(&mut g, me, noctowl, true), None);
        add_player_lock(&mut g.st, me, &EVOLVE, 1, 0);
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
    fn ability_off_from_the_lock_pass() {
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
        E::ContinuousEffect,
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
            E::Draw | E::PutIntoHand | E::PutIntoDeck | E::LeavePlay | E::Shuffle | E::Look | E::Reveal | E::CoinFlip | E::ApplyEffect | E::ContinuousEffect | E::StateCheck | E::GameEnd => ALL.contains(&k),
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
    /// which a cause-only declaration ranges over by design) are kinds whose routine asks the reader.
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
        // Neutralization Zone, Farigiraf ex, Cornerstone Mask Ogerpon ex, Shadowy Darkness Energy, Fezandipiti (coin).
        // (Milotic TWM 50's "can't be put into your opponent's hand" is a lock since the batch 7 closeout, 7b.)
        assert_eq!(n, 50);
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
                        // Read by the state check through the declaration index (`decl::fam::TABLE_STATE`).
                        assert_eq!(crate::decl::modifier_family(&p.modifier), Some(crate::decl::fam::TABLE_STATE), "{}: the recovery is never read", s.class);
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
    //! reader of the lock pass (`power_blocked`) would need the marker too.
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
        }
    }

    /// Card predicates `lock_sync`'s gate (`State::lock_synced`: the layout generation, the turn, the turn player,
    /// "Stadium and Tool cards have no effect") accounts for: the printed card.
    fn layout_pred(p: &crate::spec::value::Pred) -> bool {
        use crate::spec::value::Pred as P;
        match p {
            P::Not(q) => layout_pred(q),
            P::All(qs) | P::OneOf(qs) => qs.iter().all(layout_pred),
            P::Any | P::False | P::Pokemon | P::Basic | P::Energy | P::BasicEnergy | P::Trainer | P::Item | P::Supporter
            | P::Tool | P::Stadium | P::Name(_) | P::Tag(_) | P::Stage(_) | P::NameContains(_) | P::StageIs(_)
            | P::PrintedType(_) | P::PokemonType(_) | P::RuleBox | P::SpecialEnergy | P::PrintsAbility => true,
            _ => false,
        }
    }

    /// Spot predicates the gate accounts for: the position, the printed cards of the spot, and the type as the game
    /// checks it (changed only by attached cards and the Stadium: the layout, and the Stadium / Tool switch).
    fn layout_slot_pred(p: &crate::spec::value::SlotPred) -> bool {
        use crate::spec::value::SlotPred as S;
        match p {
            S::Not(q) => layout_slot_pred(q),
            S::All(qs) | S::OneOf(qs) => qs.iter().all(layout_slot_pred),
            S::Top(q) => layout_pred(q),
            S::Any | S::IsActive | S::IsBench | S::PrintedTypeIs(_) | S::TypeIs(_) | S::Tag(_) | S::RuleBox | S::Basic
            | S::StageIs(_) | S::Named(_) | S::IsThisPokemon | S::OnMySide | S::Evolution | S::Evolved | S::CardBelowThis(_) => true,
            _ => false,
        }
    }

    /// `lock_sync` is skipped while the layout generation, the turn, the turn player and the Stadium / Tool switch are
    /// those of the last sync (batch 8 C6): sound only while every lock source holds by what those determine. A lock
    /// reading damage, markers, Special Conditions, Prizes or hands needs its read in the gate's key first.
    #[test]
    fn lock_predicates_are_layout_determined() {
        let mut locks = Vec::new();
        for s in crate::cards::registry::SPECS.iter() {
            for ps in s.passives.iter() {
                match &ps.modifier {
                    Modifier::AbilityLock(l) => {
                        assert!(layout_pred(&l.card) && layout_pred(&l.missing), "{}: an Ability lock's card predicate outside the gate's key", s.class);
                        assert!(layout_slot_pred(&l.slot), "{}: an Ability lock's spot predicate outside the gate's key", s.class);
                        // Every probe reads the lock pass (locks over the locker) or the Stadium's effect on the spot.
                        assert!(matches!(l.probe, LockerProbe::Generic | LockerProbe::OwnPower(_) | LockerProbe::StadiumOnSlot));
                        locks.push(s.class);
                    }
                    // The Active Spot, the printed card, its Rule Box and tags.
                    Modifier::ActiveLock(ActiveLock::MidnightFluttering | ActiveLock::Initialization) => locks.push(s.class),
                    _ => {}
                }
            }
        }
        assert!(!locks.is_empty());
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
