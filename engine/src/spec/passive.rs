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
use crate::prompts::{get_target, AttachOpts, Filter, PromptKind};
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
    PreventDamage(PreventDamageSpec),
    PreventAttackEffects(PreventAttackEffectsSpec),
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
    EvolveFrom(EvolveFromSpec),
    AllowEvolve(AllowEvolveSpec),
    ConditionImmunity(ConditionImmunitySpec),
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
    /// A Stadium: when a Pokémon of the type is played, each of the player's Pokémon of that type counts as
    /// having been played the turn before (Forest of Vitality).
    PlayedTurnReset(PlayedTurnResetSpec),
    /// The Weakness of the opponent's Pokémon matching `subject` is `weakness` (Fairy Zone).
    WeaknessOverride(WeaknessOverrideSpec),
    /// An Ability lock that applies while this Pokémon is in the Active Spot, with the Ability lockers'
    /// activation order (a lock that was in effect first suppresses a later one).
    ActiveLock(ActiveLock),
    /// Heavy Baton: when the Active Pokémon this Tool is attached to, with a Retreat Cost of exactly
    /// `retreat_cost`, is Knocked Out by damage from an attack from the opponent's Pokémon, its owner moves up
    /// to `max` Basic Energy cards from it to their Benched Pokémon in any way they like.
    HeavyBaton(HeavyBatonSpec),
    // --- S3-4 appends ---
    /// The card's Pokémon can't attack unless a condition holds.
    BlockAttack(BlockAttackSpec),
}

pub struct HeavyBatonSpec {
    pub retreat_cost: usize,
    pub max: u8,
}

/// `CardFrame::stage` of Heavy Baton's prompt.
pub(crate) const HEAVY_BATON_STAGE: u8 = 0xB1;

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

pub struct PlayedTurnResetSpec {
    pub card_type: CardType,
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

/// How prevented damage is removed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreventHow {
    /// The `PutDamage` is prevented.
    Prevent,
    /// The damage is set to 0 (`DealDamage` and `PutDamage`).
    Zero,
    /// Tera: damage put on this Pokémon while it is on the Bench.
    Tera,
    /// "Flip a coin; if heads, prevent that damage" (the card's owner flips; no flip without damage).
    CoinFlip,
}

/// "Prevent all damage done to ... by attacks from ..." (vocabulary P3).
pub struct PreventDamageSpec {
    pub how: PreventHow,
    /// The damaged Pokémon.
    pub subject: SlotPred,
    /// The attacking Pokémon (always the opponent's).
    pub source: SlotPred,
    pub side: Side,
    pub guard: Cond,
}

impl PreventDamageSpec {
    pub const DEFAULT: PreventDamageSpec = PreventDamageSpec {
        how: PreventHow::Prevent,
        subject: SlotPred::Holder,
        source: SlotPred::Any,
        side: Side::Any,
        guard: Cond::True,
    };
}

/// "Prevent all effects of attacks done to this Pokémon" (vocabulary P4). The damage steps are not
/// effects (Weakness, Resistance and damage itself are never prevented here).
pub struct PreventAttackEffectsSpec {
    /// The Pokémon the effects are done to.
    pub subject: SlotPred,
    /// Also the damage counters placed by the opponent's Abilities (Hide 'n' Sneak).
    pub abilities: bool,
    /// The lock probe is made for the attacking player (today's behavior of Empoleon ex),
    /// not for the card's owner.
    pub probe_for_attacker: bool,
    /// Nothing is prevented when the attack's source slot holds no Pokémon.
    pub needs_source_pokemon: bool,
    /// Only attacks of Pokémon matching this predicate (the attacking Pokémon).
    pub attacker: SlotPred,
    /// Only Pokémon of the card's owner are protected (`Side::Owner`).
    pub side: Side,
    /// The damage steps are prevented too, unless the attack ignores effects on the
    /// Defending Pokémon (Shred): "prevent all damage from and effects of attacks".
    pub damage_too: bool,
}

impl PreventAttackEffectsSpec {
    pub const DEFAULT: PreventAttackEffectsSpec = PreventAttackEffectsSpec {
        subject: SlotPred::Holder,
        abilities: false,
        probe_for_attacker: false,
        needs_source_pokemon: true,
        attacker: SlotPred::Any,
        side: Side::Any,
        damage_too: false,
    };
}

/// What a `Prevent` passive stops.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreventWhat {
    /// Damage counters can't be moved (to other Pokémon).
    CounterMoves,
    /// Battle Cage: no damage counters on Benched Pokémon from the opponent's Pokémon's attacks
    /// and Abilities (counters moved onto them included).
    BenchCounters,
    /// The opponent's Active Pokémon can't be healed (an attack's `HealTarget` heals through `Heal`).
    HealOppActive,
    /// The opponent's Pokémon in play and their attached cards can't be put into the opponent's hand.
    MoveToHandFromOppPlay,
    // --- S3 agent 3 appends ---
    /// This card can't be put into its owner's hand or deck from the discard pile (the move
    /// takes the other cards, or is prevented when nothing is left).
    ThisCardFromDiscard,
    /// Pokémon Tools have no effect (a Stadium): every Tool effect throws unless the Stadium's
    /// effect is blocked on the Pokémon holding it.
    ToolEffects,
}

/// A prohibition on the opponent's or everyone's effects (vocabulary P5).
pub struct PreventSpec {
    pub what: PreventWhat,
}
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

/// An action a lock can stop. The card the lock's `card` / `except` predicates read is the card the action
/// uses: the card played from the hand, the Pokémon retreating, the Stadium being used.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LockedAction {
    /// Play an Item card from the hand.
    PlayItem,
    PlaySupporter,
    PlayStadium,
    /// Attach a Pokémon Tool card from the hand (a Tool put on by an effect from another zone is not
    /// "played from the hand").
    AttachTool,
    /// Attach an Energy card from the hand.
    AttachEnergy,
    /// Play a Pokémon card from the hand: onto the Bench, or onto a Pokémon to evolve it.
    PlayPokemon,
    /// Evolve a Pokémon with a card from the hand, by playing it or with Rare Candy (an Evolution card that
    /// comes from another zone doesn't pass through this action).
    Evolve,
    /// Retreat the Active Pokémon (the card is that Pokémon).
    Retreat,
    /// Use the Stadium in play (the card is the Stadium).
    UseStadium,
}

impl LockedAction {
    /// The effect kind that carries this action.
    pub const fn kind(self) -> u32 {
        use crate::effects::k;
        match self {
            LockedAction::PlayItem => k::PLAY_ITEM,
            LockedAction::PlaySupporter => k::PLAY_SUPPORTER,
            LockedAction::PlayStadium => k::PLAY_STADIUM,
            LockedAction::AttachTool => k::ATTACH_POKEMON_TOOL,
            LockedAction::AttachEnergy => k::ATTACH_ENERGY,
            LockedAction::PlayPokemon => k::PLAY_POKEMON,
            LockedAction::Evolve => k::EVOLVE,
            LockedAction::Retreat => k::RETREAT,
            LockedAction::UseStadium => k::USE_STADIUM,
        }
    }
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

/// A play or use lock, declared as data (`play_locked` evaluates it for execution and for legality): the
/// players it `binds`, the `actions` it stops, a predicate over the card the action uses (`card`, minus
/// `except`), the source's conditions (`while_`), and whether the source's Ability has to be on
/// (`ability`: a lock whose source has no Ability is off, so Jellicent ex's Item lock ends when Iron Thorns
/// ex removes its Ability). `error` is the code the blocked action fails with.
pub struct BlockUseSpec {
    pub binds: Binds,
    pub actions: &'static [LockedAction],
    pub card: Pred,
    pub except: Pred,
    pub while_: &'static [LockWhile],
    pub ability: bool,
    pub error: &'static str,
}

impl BlockUseSpec {
    /// The Stadium in play can't be used (it has no use text of its own).
    pub const USE_STADIUM: BlockUseSpec = BlockUseSpec {
        binds: Binds::Both,
        actions: &[LockedAction::UseStadium],
        card: Pred::Any,
        except: Pred::False,
        while_: &[LockWhile::CardIsSource],
        ability: false,
        error: "CANNOT_USE_STADIUM",
    };
    /// This Pokémon can't retreat while it is the Active Pokémon (the Antique Fossils).
    pub const RETREAT_THIS_ACTIVE: BlockUseSpec = BlockUseSpec {
        binds: Binds::Owner,
        actions: &[LockedAction::Retreat],
        card: Pred::Any,
        except: Pred::False,
        while_: &[LockWhile::Active, LockWhile::CardIsSource],
        ability: false,
        error: "CANNOT_RETREAT",
    };
}

const fn block_kinds(actions: &[LockedAction]) -> KindMask {
    let mut m = KindMask::EMPTY;
    let mut i = 0;
    while i < actions.len() {
        m = crate::spec::with(m, actions[i].kind());
        i += 1;
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
    /// The lock probe is made for the owner's opponent (today's behavior of Mega Gengar ex).
    pub probe_opponent: bool,
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
        probe_opponent: false,
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
/// The Pokémon also evolves from `names` (Eevee ex's Rainbow DNA); only a card matching `only`
/// may be played onto it.
pub struct EvolveFromSpec {
    pub names: &'static [&'static str],
    pub only: Pred,
}
/// "Can evolve during your first turn or the turn you play it": the Pokémon's played turn is the
/// turn before, and it may evolve on the first turn, while it satisfies `subject`.
pub struct AllowEvolveSpec {
    pub subject: SlotPred,
}
/// "Can't be affected by Special Conditions" (vocabulary P20).
pub struct ConditionImmunitySpec {
    /// The conditions (empty: all of them).
    pub conds: &'static [SpecialCondition],
    pub subject: SlotPred,
    /// The conditions are removed from effects that would add them.
    pub prevent: bool,
    /// Cleared whenever the table state is checked (Festival Grounds).
    pub sweep: bool,
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
        Modifier::BlockUse(b) => block_kinds(b.actions),
        Modifier::ProvidesEnergy(_) | Modifier::ProvidesEnergyBoost(_) => mask(&[k::CHECK_PROVIDED_ENERGY]),
        Modifier::AttachGuard(_) => mask(&[k::ATTACH_ENERGY, k::CHECK_TABLE_STATE]),
        Modifier::ConditionImmunity(c) => match (c.prevent, c.sweep) {
            (true, true) => mask(&[k::ADD_SPECIAL_CONDITIONS, k::ADD_SPECIAL_CONDITIONS_POWER, k::CHECK_TABLE_STATE]),
            (true, false) => mask(&[k::ADD_SPECIAL_CONDITIONS, k::ADD_SPECIAL_CONDITIONS_POWER]),
            _ => mask(&[k::CHECK_TABLE_STATE]),
        },
        Modifier::AbilityLock(_) => mask(&[k::CHECK_POKEMON_POWERS, k::POWER]),
        Modifier::Prevent(p) => match p.what {
            PreventWhat::CounterMoves => mask(&[k::MOVE_DAMAGE_COUNTERS, k::MOVE_COUNTERS]),
            PreventWhat::BenchCounters => mask(&[k::MOVE_COUNTERS, k::PUT_COUNTERS, k::PLACE_DAMAGE_COUNTERS]),
            PreventWhat::HealOppActive => mask(&[k::HEAL]),
            PreventWhat::MoveToHandFromOppPlay => mask(&[k::MOVE_CARDS]),
            PreventWhat::ThisCardFromDiscard => mask(&[k::MOVE_CARDS]),
            PreventWhat::ToolEffects => mask(&[k::TOOL]),
        },
        Modifier::PreventAttackEffects(_) => HIDE_N_SNEAK_MASK,
        Modifier::PrizeAdjust(_) | Modifier::PrizeAdjustOnce(_) => mask(&[k::KNOCK_OUT]),
        Modifier::TypeOverride(_) => mask(&[k::CHECK_POKEMON_TYPE]),
        Modifier::WeaknessOverride(_) => mask(&[k::CHECK_POKEMON_STATS]),
        Modifier::HeavyBaton(_) => mask(&[k::KNOCK_OUT, k::PUT_DAMAGE]),
        Modifier::ActiveLock(ActiveLock::MidnightFluttering) => mask(&[k::CHECK_POKEMON_POWERS, k::POWER]),
        Modifier::ActiveLock(ActiveLock::Initialization) => mask(&[k::CHECK_POKEMON_POWERS, k::POWER, k::EFFECT_OF_ABILITY]),
        Modifier::PlayedTurnReset(_) => mask(&[k::PLAY_POKEMON]),
        Modifier::GrantAttacks(_) => mask(&[k::CHECK_POKEMON_ATTACKS]),
        Modifier::EvolveFrom(_) => mask(&[k::CHECK_TABLE_STATE, k::PLAY_POKEMON]),
        Modifier::AttackCost(_) => mask(&[k::CHECK_ATTACK_COST]),
        Modifier::BlockAttack(b) => match b.on {
            AttackBlockOn::ActiveAttack => mask(&[k::ATTACK]),
            AttackBlockOn::UseAttack => mask(&[k::USE_ATTACK]),
        },
        Modifier::RetreatCost(_) => mask(&[k::CHECK_RETREAT_COST]),
        Modifier::BenchSize(_) => mask(&[k::CHECK_TABLE_STATE]),
        Modifier::AllowEvolve(_) => mask(&[k::CHECK_POKEMON_PLAYED_TURN]),
        Modifier::PreventDamage(p) => match p.how {
            PreventHow::Zero => mask(&[k::DEAL_DAMAGE, k::PUT_DAMAGE]),
            _ => mask(&[k::PUT_DAMAGE]),
        },
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
    for p in 0..2 {
        for s in g.st.players[p].in_play().iter() {
            if f(g.st.slot(p, *s), p, *s) {
                return Some(SlotRef::new(p, *s));
            }
        }
    }
    None
}

pub(crate) fn locate(g: &Game, me: CardId, origin: RuleSource) -> Option<Located> {
    match origin {
        RuleSource::Tool => slot_where(g, |sl, _, _| sl.tools.contains(me)).map(|s| Located { owner: s.p as usize, held: Some(s) }),
        RuleSource::Energy => slot_where(g, |sl, _, _| sl.cards.contains(me) && !sl.tools.contains(me)).map(|s| Located { owner: s.p as usize, held: Some(s) }),
        RuleSource::Ability => slot_where(g, |_, p, s| g.st.slot_pokemon(p, s) == Some(me)).map(|s| Located { owner: s.p as usize, held: Some(s) }),
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

fn guard_ok(g: &Game, me: CardId, owner: usize, guard: &Cond) -> bool {
    let f = run::Frame::new(run::Prog::Play, run::Phase::Use, 0, owner);
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
        Modifier::PreventDamage(d) => prevent_damage(g, me, e, ps.origin, d),
        Modifier::BlockUse(_) => {
            // The same query legality asks (`play_locked`), restricted to this source.
            let Some((p, card, action)) = effect_action(g, e) else { return Ok(()) };
            match play_locked_by(g, me, p, card, action) {
                Some(code) => crate::bail!(code),
                None => Ok(()),
            }
        }
        Modifier::ProvidesEnergy(pe) => provides_energy(g, me, e, pe),
        Modifier::ProvidesEnergyBoost(b) => provides_energy_boost(g, me, e, ps.origin, b),
        Modifier::AttachGuard(a) => attach_guard(g, me, e, a),
        Modifier::AttackCost(c) => attack_cost(g, me, e, ps.origin, c),
        Modifier::ConditionImmunity(c) => condition_immunity(g, me, e, ps.origin, c),
        Modifier::AbilityLock(l) => ability_lock(g, me, e, l),
        Modifier::Prevent(p) => prevent(g, me, e, ps.origin, p),
        Modifier::PreventAttackEffects(d) => prevent_attack_effects(g, me, e, ps.origin, d),
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
        Modifier::PlayedTurnReset(r) => played_turn_reset(g, me, e, ps.origin, r),
        Modifier::ActiveLock(l) => active_lock(g, me, e, *l),
        Modifier::HeavyBaton(h) => heavy_baton(g, me, e, h),
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
            let (fx, _) = g.run_fx(Effect::EffectOfAbility { p: at.owner as u8, power: crate::effects::PowerRef { card: me, index: 0 }, card: me, target: Some(target) })?;
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
        Modifier::EvolveFrom(d) => evolve_from(g, me, e, d),
        Modifier::RetreatCost(c) => retreat_cost(g, me, e, ps.origin, c),
        Modifier::BenchSize(d) => bench_size(g, me, e, ps.origin, d),
        Modifier::AllowEvolve(d) => allow_evolve(g, me, e, ps.origin, d),
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
    if !slot_pred_m(g, me, target, subject)? || blocked(g, me, origin, at, Some(target)) || !guard_ok_m(g, me, at.owner, guard)? {
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
    if !slot_pred_m(g, me, source, &d.attacker)? || !guard_ok(g, me, at.owner, &d.guard) {
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
    if !guard_ok(g, me, at.owner, guard) || !slot_pred_m(g, me, b.source, source)? {
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
        if g.prevented(e) || damage_now <= 0 || crate::engine::attack::should_prevent_attack_damage(g, b.target, b.source) {
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
        let t = b.target;
        let owner = t.p;
        move_cards(g, ListRef::Slot(t.p, t.s), ListRef::Discard(owner), &[me], NO_CARD)?;
    }
    Ok(())
}

fn prevent_damage(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &PreventDamageSpec) -> R {
    if d.how == PreventHow::Tera {
        // `TERA_RULE`: only for the card on top of its Pokémon.
        if let Effect::PutDamage { .. } = *g.e(e) {
            tera_rule(g, e, me);
        }
        return Ok(());
    }
    if (d.how == PreventHow::Prevent || d.how == PreventHow::CoinFlip) && !matches!(*g.e(e), Effect::PutDamage { .. }) {
        return Ok(());
    }
    let Some((_, at)) = damage_taken_prelude(g, me, e, origin, d.side, &d.subject, &d.source, &d.guard, false)? else { return Ok(()) };
    if d.how == PreventHow::CoinFlip {
        let damage = match *g.e(e) {
            Effect::PutDamage { damage, .. } => damage,
            _ => 0,
        };
        if damage <= 0 {
            return Ok(());
        }
        let (c, _) = g.run_fx(Effect::CoinFlip { p: at.owner as u8, callback: None, result: None, skip_reflip_stadium: false, skip_reflip_tool: false })?;
        if let Effect::CoinFlip { result: Some(false), .. } = c {
            return Ok(());
        }
        g.set_prevent(e, true);
        return Ok(());
    }
    match d.how {
        PreventHow::Prevent => g.set_prevent(e, true),
        PreventHow::CoinFlip | PreventHow::Tera => {}
        _ => match g.e_mut(e) {
            Effect::DealDamage { damage, .. } | Effect::PutDamage { damage, .. } => *damage = 0,
            _ => {}
        },
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

fn attach_guard(g: &mut Game, me: CardId, e: EffId, a: &AttachGuardSpec) -> R {
    match *g.e(e) {
        Effect::AttachEnergy { card, target, .. } if card == me => {
            if !slot_pred_m(g, me, target, &a.allow)? {
                crate::bail!("CANNOT_PLAY_THIS_CARD");
            }
        }
        Effect::CheckTableState { .. } => {
            for p in 0..2usize {
                for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                    let t = SlotRef::new(p, s);
                    if !g.st.slot(p, s).cards.contains(me) || is_special_energy_blocked(g, p, me, t, false) {
                        continue;
                    }
                    if g.st.slot_pokemon(p, s).is_some() && !slot_pred_m(g, me, t, &a.allow)? {
                        move_cards(g, t.list(), ListRef::Discard(p as u8), &[me], me)?;
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
        let f = run::Frame::new(run::Prog::Play, run::Phase::Use, 0, at.owner);
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
        let f = run::Frame::new(run::Prog::Play, run::Phase::Use, 0, at.owner);
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

fn condition_immunity(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, c: &ConditionImmunitySpec) -> R {
    if c.prevent {
        let (target, conditions) = match *g.e(e) {
            Effect::AddSpecialConditions { b, conditions, .. } => (b.target, conditions),
            Effect::AddSpecialConditionsPower { target, conditions, .. } => (target, conditions),
            _ => (SlotRef::new(0, 0), SVec::new()),
        };
        if !conditions.is_empty() {
            let Some(at) = locate(g, me, origin) else { return Ok(()) };
            let hit = |x: &u8| c.conds.is_empty() || c.conds.iter().any(|y| *y as u8 == *x);
            if conditions.iter().any(hit) && slot_pred_m(g, me, target, &c.subject)? && !blocked(g, me, origin, at, Some(target)) {
                let mut remaining = conditions;
                remaining.retain(|x| !hit(x));
                if remaining.is_empty() {
                    g.set_prevent(e, true);
                } else {
                    match g.e_mut(e) {
                        Effect::AddSpecialConditions { conditions, .. } | Effect::AddSpecialConditionsPower { conditions, .. } => *conditions = remaining,
                        _ => {}
                    }
                }
            }
            return Ok(());
        }
    }
    if c.sweep && matches!(*g.e(e), Effect::CheckTableState { .. }) {
        let Some(at) = locate(g, me, origin) else { return Ok(()) };
        for p in 0..2usize {
            for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                let t = SlotRef::new(p, s);
                if g.st.slot(p, s).special_conditions.is_empty() || !slot_pred_m(g, me, t, &c.subject)? || blocked(g, me, origin, at, Some(t)) {
                    continue;
                }
                // `clearAllSpecialConditions()`: removes the five conditions.
                let sc = &mut g.st.players[p].slots[s as usize].special_conditions;
                for x in [SpecialCondition::Poisoned, SpecialCondition::Asleep, SpecialCondition::Burned, SpecialCondition::Confused, SpecialCondition::Paralyzed] {
                    if c.conds.is_empty() || c.conds.contains(&x) {
                        sc.retain(|y| *y != x as u8);
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

fn prevent(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, p: &PreventSpec) -> R {
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    match (p.what, *g.e(e)) {
        (PreventWhat::CounterMoves, Effect::MoveDamageCounters { .. } | Effect::MoveCounters { .. }) => {
            if !blocked(g, me, origin, at, None) {
                g.set_prevent(e, true);
            }
        }
        // Counters moved onto a Benched Pokémon by the opponent's effect: they are taken off the source and
        // vanish (ruling 2257).
        (PreventWhat::BenchCounters, Effect::MoveCounters { b, .. }) => {
            if bench_target_prevented(g, me, b.player as usize, b.target) {
                g.set_prevent(e, true);
            }
        }
        (PreventWhat::BenchCounters, Effect::PutCounters { b, .. }) => {
            if bench_target_prevented(g, me, b.source.p as usize, b.target) {
                g.set_prevent(e, true);
            }
        }
        (PreventWhat::BenchCounters, Effect::PlaceDamageCounters { target, source, .. }) => {
            if source == NO_CARD {
                return Ok(());
            }
            if let Some((owner, _)) = g.st.find_pokemon_slot(source) {
                if bench_target_prevented(g, me, owner, target) {
                    g.set_prevent(e, true);
                }
            }
        }
        (PreventWhat::HealOppActive, Effect::Heal { target, .. }) => {
            let o = 1 - at.owner;
            if target.p as usize == o && target.s == g.st.players[o].active && !blocked(g, me, origin, at, None) {
                g.set_prevent(e, true);
            }
        }
        (PreventWhat::MoveToHandFromOppPlay, Effect::MoveCards { source, destination, .. }) => {
            let ListRef::Slot(sq, ss) = source else { return Ok(()) };
            let (sq, opp) = (sq as usize, 1 - at.owner);
            if destination == ListRef::Hand(opp as u8) && sq == opp && g.st.slot_pokemon(opp, ss).is_some() && !blocked(g, me, origin, at, None) {
                g.set_prevent(e, true);
            }
        }
        (PreventWhat::ThisCardFromDiscard, Effect::MoveCards { source, destination, cards, count, .. }) => {
            // Runs for any discard pile holding this card (no in-play check).
            for q in 0..2usize {
                if source != ListRef::Discard(q as u8) || !g.st.players[q].discard.iter().any(|c| c == me) {
                    continue;
                }
                if destination != ListRef::Hand(q as u8) && destination != ListRef::Deck(q as u8) {
                    continue;
                }
                let v: Vec<CardId>;
                let new_count;
                if let Some(cs) = cards {
                    if !cs.iter().any(|c| c == me) {
                        continue;
                    }
                    v = cs.iter().filter(|c| *c != me).collect();
                    new_count = count;
                } else if let Some(n) = count {
                    v = g.st.players[q].discard.iter().filter(|c| *c != me).take(n.max(0) as usize).collect();
                    new_count = None;
                } else {
                    v = g.st.players[q].discard.iter().filter(|c| *c != me).collect();
                    new_count = None;
                }
                let prevent = v.is_empty();
                if let Effect::MoveCards { cards, count, .. } = g.e_mut(e) {
                    *cards = Some(List::from_slice(&v));
                    *count = new_count;
                }
                if prevent {
                    g.set_prevent(e, true);
                }
            }
        }
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
// Attack effects

/// Every attack-effect kind (`AtkBase` effects) except the damage steps, plus the counters
/// placed by Abilities: the effect kinds Hide 'n' Sneak and Mist Energy prevent.
pub const HIDE_N_SNEAK_KINDS: [u32; 37] = [
    crate::effects::k::SELF_PREVENT_RETREAT,
    crate::effects::k::DISCARD_ATTACKER_ENERGY_IF_KO,
    crate::effects::k::APPLY_WEAKNESS,
    crate::effects::k::DEAL_DAMAGE,
    crate::effects::k::PUT_DAMAGE,
    crate::effects::k::AFTER_DAMAGE,
    crate::effects::k::PUT_COUNTERS,
    crate::effects::k::KNOCK_OUT_OPPONENT,
    crate::effects::k::KNOCK_OUT_PLAYER,
    crate::effects::k::RETALIATE_ON_DAMAGE,
    crate::effects::k::RETALIATE_DAMAGE,
    crate::effects::k::DISCARD_CARDS,
    crate::effects::k::CARDS_TO_HAND,
    crate::effects::k::GUST_OPPONENT_BENCH,
    crate::effects::k::MOVE_OPPONENT_ENERGY,
    crate::effects::k::ADD_MARKER,
    crate::effects::k::ADD_SPECIAL_CONDITIONS,
    crate::effects::k::ADD_SPECIAL_CONDITIONS_POWER,
    crate::effects::k::REMOVE_SPECIAL_CONDITIONS,
    crate::effects::k::HEAL_TARGET,
    crate::effects::k::PLAY_LOCK,
    crate::effects::k::PREVENT_RETREAT,
    crate::effects::k::OPPONENT_POKEMON_CANNOT_USE_ATTACK,
    crate::effects::k::PREVENT_ATTACK_UNTIL_LEAVES_ACTIVE,
    crate::effects::k::DEFENDING_POKEMON_TAKES_MORE_DAMAGE,
    crate::effects::k::REDUCE_DAMAGE,
    crate::effects::k::SWITCH_OUT_OPPONENTS_ACTIVE,
    crate::effects::k::PLACE_DAMAGE_COUNTERS,
    crate::effects::k::PREVENT_DAMAGE,
    crate::effects::k::PREVENT_EFFECTS_OF_ATTACKS,
    crate::effects::k::THIS_POKEMON_HAS_NO_WEAKNESS,
    crate::effects::k::OPPONENT_POKEMON_CANNOT_ATTACK_NEXT_TURN,
    crate::effects::k::INCREASE_ATTACK_COST_NEXT_TURN,
    crate::effects::k::INCREASE_RETREAT_COST_NEXT_TURN,
    crate::effects::k::COIN_FLIP_CANCEL_TRAINER_PLAY,
    crate::effects::k::MOVE_COUNTERS,
    crate::effects::k::DEVOLVE,
];

pub const HIDE_N_SNEAK_MASK: KindMask = mask(&HIDE_N_SNEAK_KINDS);

fn prevent_attack_effects(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &PreventAttackEffectsSpec) -> R {
    if let Some(b) = g.e(e).atk_base().copied() {
        let t = b.target;
        let Some(at) = locate(g, me, origin) else { return Ok(()) };
        if !slot_pred_m(g, me, t, &d.subject)? || (d.side == Side::Owner && at.owner != t.p as usize) {
            return Ok(());
        }
        let probe_as = if d.probe_for_attacker { Located { owner: b.player as usize, ..at } } else { at };
        if blocked(g, me, origin, probe_as, Some(t)) {
            return Ok(());
        }
        let attacker = if d.probe_for_attacker { b.source.p } else { b.player };
        if attacker == t.p {
            return Ok(());
        }
        let damage_step = matches!(*g.e(e), Effect::ApplyWeakness { .. } | Effect::PutDamage { .. } | Effect::DealDamage { .. });
        if damage_step && !d.damage_too {
            return Ok(());
        }
        if d.damage_too && (damage_step || matches!(*g.e(e), Effect::AfterDamage { .. })) && ignores_defender_effects(g, &b) {
            return Ok(());
        }
        if !matches!(d.attacker, SlotPred::Any) && !slot_pred_m(g, me, b.source, &d.attacker)? {
            return Ok(());
        }
        if d.needs_source_pokemon && g.st.slot_pokemon(b.source.p as usize, b.source.s).is_none() {
            return Ok(());
        }
        g.set_prevent(e, true);
        return Ok(());
    }
    if d.abilities {
        // A Special Condition added by the Ability of an opposing Pokémon (an effect built by a Trainer or
        // by the owner's own Pokémon doesn't count).
        if let Effect::AddSpecialConditionsPower { target, source, .. } = *g.e(e) {
            let Some(at) = locate(g, me, origin) else { return Ok(()) };
            if !slot_pred_m(g, me, target, &d.subject)? || blocked(g, me, origin, at, Some(target)) {
                return Ok(());
            }
            if g.st.cdef(source).is_pokemon() {
                if let Some((q, _)) = g.st.find_pokemon_slot(source) {
                    if q != target.p as usize {
                        g.set_prevent(e, true);
                    }
                }
            }
            return Ok(());
        }
        if let Effect::PlaceDamageCounters { p, target, source, .. } = *g.e(e) {
            let Some(at) = locate(g, me, origin) else { return Ok(()) };
            if !slot_pred_m(g, me, target, &d.subject)? || blocked(g, me, origin, at, Some(target)) {
                return Ok(());
            }
            if p as usize == target.p as usize || source == NO_CARD {
                return Ok(());
            }
            match g.st.find_pokemon_slot(source) {
                Some((q, _)) if q == p as usize => {}
                _ => return Ok(()),
            }
            g.set_prevent(e, true);
        }
    }
    Ok(())
}

/// Hide 'n' Sneak: prevent all effects of the opponent's Pokémon's attacks and Abilities done to
/// this Pokémon (damage is not an effect).
pub const HIDE_N_SNEAK: PreventAttackEffectsSpec = PreventAttackEffectsSpec {
    subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
    abilities: true,
    probe_for_attacker: false,
    needs_source_pokemon: false,
    attacker: SlotPred::Any,
    side: Side::Any,
    damage_too: false,
};

/// "Prevent all effects of attacks used by your opponent's Pokémon done to this Pokémon" (a Fossil's
/// Protective Cover): Hide 'n' Sneak without the Abilities part.
pub const HIDE_N_SNEAK_ATTACKS: PreventAttackEffectsSpec = PreventAttackEffectsSpec {
    subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
    abilities: false,
    probe_for_attacker: false,
    needs_source_pokemon: true,
    attacker: SlotPred::Any,
    side: Side::Any,
    damage_too: false,
};

// ---------------------------------------------------------------------------
// Prizes, evolution, attacks

fn prize_adjust(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &PrizeAdjustSpec) -> R {
    let (p, target) = match *g.e(e) {
        Effect::KnockOut { p, target, .. } => (p as usize, target),
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
    let probe_at = if d.probe_opponent { Located { owner: 1 - p, ..at } } else { at };
    if blocked(g, me, origin, probe_at, Some(target)) {
        return Ok(());
    }
    let by_damage = g.knocked_out_by_attack_damage(p, target);
    if d.by_attack_damage && by_damage.is_none() {
        return Ok(());
    }
    if let Some((_, Some(src))) = by_damage {
        if !slot_pred_m(g, me, src, &d.attacker)? {
            return Ok(());
        }
    }
    if !guard_ok(g, me, at.owner, &d.guard) {
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

fn evolve_from(g: &mut Game, me: CardId, e: EffId, d: &EvolveFromSpec) -> R {
    static NONE: &[&str] = &[];
    match *g.e(e) {
        Effect::CheckTableState { .. } => {
            let v = match g.st.find_pokemon_slot(me) {
                None => NONE,
                Some((owner, _)) => {
                    if is_ability_blocked(g, owner, me, None) {
                        NONE
                    } else {
                        d.names
                    }
                }
            };
            g.st.cards[me as usize].evolves_from_base = Some(v);
        }
        // Only a card matching `only` can evolve this Pokémon.
        Effect::PlayPokemon { card, target, .. } => {
            if g.st.slot_pokemon(target.p as usize, target.s) == Some(me) {
                let def = g.st.cdef(card);
                if d.names.iter().any(|n| *n == def.evolves_from) && !pred(g, card, &d.only) {
                    crate::bail!("INVALID_TARGET");
                }
            }
        }
        _ => {}
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
// Play and use locks (`Modifier::BlockUse`), declared as data
//
// `play_locked` is the one query: execution (the passive's handler, per lock source, via
// `play_locked_by`) and legality (all sources) read the same declarations, so they can't drift.

/// The action an effect carries when a lock can stop it: (the acting player, the card the action uses,
/// the action). Attaching a Tool or an Energy counts only when the card is in the hand ("played from the
/// hand"; a card put on by an effect from the deck or the discard pile isn't).
pub(crate) fn effect_action(g: &Game, e: EffId) -> Option<(usize, CardId, LockedAction)> {
    Some(match *g.e(e) {
        Effect::PlayItem { p, card, .. } => (p as usize, card, LockedAction::PlayItem),
        Effect::PlaySupporter { p, card, .. } => (p as usize, card, LockedAction::PlaySupporter),
        Effect::PlayStadium { p, card } => (p as usize, card, LockedAction::PlayStadium),
        Effect::AttachPokemonTool { p, card, .. } if g.st.players[p as usize].hand.contains(card) => (p as usize, card, LockedAction::AttachTool),
        Effect::AttachEnergy { p, card, .. } if g.st.players[p as usize].hand.contains(card) => (p as usize, card, LockedAction::AttachEnergy),
        Effect::PlayPokemon { p, card, .. } => (p as usize, card, LockedAction::PlayPokemon),
        Effect::Evolve { p, card, .. } => (p as usize, card, LockedAction::Evolve),
        Effect::Retreat { p, .. } => (p as usize, g.st.active_pokemon(p as usize)?, LockedAction::Retreat),
        Effect::UseStadium { p, stadium } => (p as usize, stadium, LockedAction::UseStadium),
        _ => return None,
    })
}

/// Is player `p` stopped from doing `action` with `card` (the card played from the hand, the retreating
/// Pokémon, the Stadium being used)? The error code of the first lock that stops it, in the order the
/// effect reaches the lock sources (the game's propagation order); `None` when no lock does.
///
/// Walks every card with a `BlockUse` declaration for the action and evaluates it (`lock_blocks`). A lock
/// whose source has no Ability is off. A card in the hand is judged by its printed data. Locks that last
/// (`cannot_play_item_cards` and the like, set by attacks) are player flags checked by the core reducers,
/// not here.
pub fn play_locked(g: &mut Game, p: usize, card: CardId, action: LockedAction) -> Option<&'static str> {
    let kind = action.kind();
    if !g.kinds_present.has(kind) {
        return None;
    }
    // The order only depends on the kind (every action's effect ranks cards by their super type).
    let probe = Effect::PlayItem { p: p as u8, card, target: None };
    let order = g.propagation_order(&probe, kind);
    for c in order.iter().copied() {
        if let Some(code) = play_locked_by(g, c, p, card, action) {
            return Some(code);
        }
    }
    None
}

/// `play_locked` for one lock source: the passive handler of `me` calls this, at the point the effect
/// reaches it.
pub(crate) fn play_locked_by(g: &mut Game, me: CardId, p: usize, card: CardId, action: LockedAction) -> Option<&'static str> {
    let passives: &'static [Passive] = crate::cards::spec_for(g.st.cards[me as usize].def).map_or(&[], |s| s.passives);
    for ps in passives {
        if let Modifier::BlockUse(b) = &ps.modifier {
            if let Some(code) = lock_blocks(g, me, ps.origin, b, p, card, action) {
                return Some(code);
            }
        }
    }
    None
}

/// Does the lock `b`, declared by `me`, stop player `p` from doing `action` with `card`?
fn lock_blocks(g: &mut Game, me: CardId, origin: RuleSource, b: &BlockUseSpec, p: usize, card: CardId, action: LockedAction) -> Option<&'static str> {
    if !b.actions.contains(&action) {
        return None;
    }
    let at = locate(g, me, origin)?;
    let binds = match b.binds {
        Binds::Opponent => p == 1 - at.owner,
        Binds::Owner => p == at.owner,
        Binds::Both => true,
    };
    if !binds {
        return None;
    }
    for w in b.while_ {
        let on = match w {
            LockWhile::Active => g.st.active_pokemon(at.owner) == Some(me),
            LockWhile::HasTool => at.held.map_or(false, |h| !g.st.slot(h.p as usize, h.s).tools.is_empty()),
            LockWhile::CardIsSource => card == me,
        };
        if !on {
            return None;
        }
    }
    // The card is judged by its printed data.
    if !pred(g, card, &b.card) || pred(g, card, &b.except) {
        return None;
    }
    if b.ability && !source_ability_on(g, at.owner, me) {
        return None;
    }
    Some(b.error)
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
            survive_on_ten_on_coin_flip(g, e, owner)?
        }
        SurviveKind::IfFullHp => {
            if g.st.slot(owner, t.s).damage != 0 {
                return Ok(());
            }
            let hp = crate::engine::check::check_hp(g, owner, t.s)?;
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

fn played_turn_reset(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, r: &PlayedTurnResetSpec) -> R {
    let (p, card, target) = match *g.e(e) {
        Effect::PlayPokemon { p, card, target, .. } => (p as usize, card, target),
        _ => return Ok(()),
    };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    // Not during a player's first turn; only when a Pokémon of the type is played.
    if g.st.turn <= 2 || !g.st.cdef(card).card_type.contains(&r.card_type) || blocked(g, me, origin, at, Some(target)) {
        return Ok(());
    }
    for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let sr = SlotRef::new(p, s);
        if blocked(g, me, origin, at, Some(sr)) {
            continue;
        }
        let (t, _) = g.run_fx(Effect::CheckPokemonType { target: sr, card_types: crate::engine::game_effect::pokemon_types(g, sr) })?;
        let of_type = match t {
            Effect::CheckPokemonType { card_types, .. } => card_types.contains(&r.card_type),
            _ => false,
        };
        if of_type {
            g.st.players[p].slots[s as usize].pokemon_played_turn = g.st.turn as i32 - 1;
        }
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
    let active = g.st.players[p].active;
    if !a.first_turn || !g.st.slot(p, active).cards.contains(me) || g.st.turn != 1 {
        return Ok(());
    }
    let at = Located { owner: p, held: None };
    if blocked(g, me, origin, at, None) {
        return Ok(());
    }
    // A copy-attack clone carries its own flag (nothing reads it).
    if !attack.is_clone() {
        g.st.cards[attack.card as usize].attack_first_turn |= 1u8 << attack.idx();
    }
    Ok(())
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
// Precedence between locks (RULES.md, 2026-10-08)
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

/// Keep the take-hold stamps up to date after the board changed: a lock source that holds (in place, its
/// Ability on) and has no stamp gets the next one, the turn player's first; one that doesn't hold loses its
/// stamp. Releasing a lock that was turned off can let the other take hold, so it repeats until nothing
/// changes.
pub(crate) fn lock_sync(g: &mut Game) {
    if g.lock_syncing || !g.kinds_present.has(crate::effects::k::CHECK_POKEMON_POWERS) {
        return;
    }
    g.lock_syncing = true;
    for _ in 0..4 {
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
        for c in 0..g.st.n_cards {
            if g.st.cards[c as usize].lock_stamp != 0 && !sources.contains(&c) {
                sources.push(c);
            }
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
            let Some(my_list) = g.st.locate(me) else { crate::bail!("INVALID_GAME_STATE") };
            let Some(owner) = my_list.owner() else { crate::bail!("INVALID_GAME_STATE") };
            if g.st.active_pokemon(owner) != Some(me) {
                return Ok(false);
            }
            let opponent = 1 - owner;
            let Some(target_list) = g.st.locate(card) else { crate::bail!("INVALID_GAME_STATE") };
            if target_list != ListRef::Slot(opponent as u8, g.st.players[opponent].active) {
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
            // A card in the hand is judged by its printed data (RULES.md): only Pokémon in play are locked.
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
                return Ok(match g.run_fx(Effect::EffectOfAbility { p: locker_owner as u8, power: own, card: me, target: Some(slot) }) {
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
fn guard_ok_m(g: &mut Game, me: CardId, owner: usize, guard: &Cond) -> R<bool> {
    let f = run::Frame::new(run::Prog::Play, run::Phase::Use, 0, owner);
    cond_m(g, me, &f, guard)
}

/// A Benched Pokémon whose counters come from the opponent's Pokémon (Battle Cage).
fn bench_target_prevented(g: &mut Game, me: CardId, source_owner: usize, t: SlotRef) -> bool {
    let owner = t.p as usize;
    if source_owner != 1 - owner || t.s == g.st.players[owner].active {
        return false;
    }
    !is_stadium_effect_blocked(g, owner, t, me)
}

fn bench_size(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &BenchSizeSpec) -> R {
    let Effect::CheckTableState { bench_sizes } = *g.e(e) else { return Ok(()) };
    if locate(g, me, origin).is_none() {
        return Ok(());
    }
    let mut sizes = bench_sizes;
    for (p, size) in sizes.iter_mut().enumerate() {
        if guard_ok(g, me, p, &d.guard) {
            *size = d.size;
        }
    }
    if let Effect::CheckTableState { bench_sizes } = g.e_mut(e) {
        *bench_sizes = sizes;
    }
    Ok(())
}

fn allow_evolve(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, d: &AllowEvolveSpec) -> R {
    let Effect::CheckPokemonPlayedTurn { p, target, .. } = *g.e(e) else { return Ok(()) };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if target.p as usize != p as usize || at.owner != p as usize || !slot_pred_m(g, me, target, &d.subject)? || blocked(g, me, origin, at, Some(target)) {
        return Ok(());
    }
    let turn = g.st.turn as i32;
    if let Effect::CheckPokemonPlayedTurn { pokemon_played_turn, can_evolve_on_first_turn, .. } = g.e_mut(e) {
        *pokemon_played_turn = turn - 1;
        *can_evolve_on_first_turn = true;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Heavy Baton

fn heavy_baton(g: &mut Game, me: CardId, e: EffId, h: &HeavyBatonSpec) -> R {
    use crate::markers::{intern, SourceType, TargetScope};
    let baton = intern("HEAVY_BATON_MARKER");
    let active_marker = intern("HEAVY_BATON_ACTIVE_MARKER");
    // The criteria are checked when the damage is dealt (ruling 1547): an attack that moves the Pokémon to the
    // Bench before the Knock Out is checked doesn't stop Heavy Baton. The latest damage from an opponent's
    // attack decides; the marker is consumed by the Knock Out.
    if let Effect::PutDamage { b, damage, .. } = *g.e(e) {
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).tools.contains(me) {
            let owner = t.p as usize;
            if g.st.phase == GamePhase::Attack && b.player as usize != owner {
                g.st.players[owner].slots[t.s as usize].marker.remove_from(active_marker, me);
                if g.st.players[owner].active == t.s && !g.prevented(e) && damage > 0 && !is_tool_blocked(g, owner, me) {
                    let cost = crate::engine::retreat::check_retreat_cost_base(g, owner);
                    let (rc, _) = g.run_fx(Effect::CheckRetreatCost { p: owner as u8, cost, no_cost: false, reduction: 0 })?;
                    if matches!(rc, Effect::CheckRetreatCost { cost, .. } if cost.len() == h.retreat_cost) {
                        // A Trainer's effect on the Pokémon: it stays when the Pokémon moves to the Bench.
                        g.st.players[owner].slots[t.s as usize].marker.add(active_marker, me, SourceType::Trainer, TargetScope::Pokemon);
                    }
                }
            }
        }
        return Ok(());
    }
    let (p, t) = match *g.e(e) {
        Effect::KnockOut { p, target, .. } => (p as usize, target),
        _ => return Ok(()),
    };
    if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
        return Ok(());
    }
    let opponent = 1 - p;
    if is_tool_blocked(g, p, me) {
        return Ok(());
    }
    if g.st.phase != GamePhase::Attack || g.st.active_player as usize != opponent {
        return Ok(());
    }
    if g.st.slot(t.p as usize, t.s).marker.has(baton) {
        return Ok(());
    }
    // Only when it was damaged by an attack while in the Active Spot with the Retreat Cost and Knocked Out by
    // damage from an attack.
    let was_active = g.st.slot(t.p as usize, t.s).marker.has_from(active_marker, me);
    g.st.players[t.p as usize].slots[t.s as usize].marker.remove_from(active_marker, me);
    if !was_active || !g.st.players[p].marker.has(crate::markers::DAMAGE_DEALT_MARKER) {
        return Ok(());
    }
    if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
        return Ok(());
    }
    let energy: Vec<CardId> = g
        .st
        .slot(t.p as usize, t.s)
        .cards
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        })
        .collect();
    if energy.is_empty() {
        return Ok(());
    }
    // Nothing to move the Energy to without a Benched Pokémon.
    if !g.st.players[p].bench.iter().any(|&b| !g.st.players[p].slots[b as usize].cards.is_empty()) {
        return Ok(());
    }
    g.st.players[t.p as usize].slots[t.s as usize].marker.add(baton, me, SourceType::None, TargetScope::None);
    let temp = g.alloc_temp(&energy);
    let mut o = AttachOpts::new(energy.len() as u8);
    // "up to" from a public zone: at least 1, no cancel (rulings 1607/1778/1853)
    o.allow_cancel = false;
    o.min = 1;
    o.max = h.max;
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let mut f = crate::cards::CardFrame::at(HEAVY_BATON_STAGE);
    f.a[0] = p as i32;
    f.a[1] = t.s as i32;
    let id = g.player_id(p);
    g.prompt(id, "ATTACH_ENERGY_TO_BENCH", PromptKind::AttachEnergy { cards: temp, player_type: PlayerType::BottomPlayer, slots, filter, o }, crate::game::Cont::Card { card: me, frame: f });
    Ok(())
}

/// The answer of Heavy Baton's prompt: the Energy moves from the owner's discard pile (the Knock Out has
/// already discarded the Pokémon) to the chosen Benched Pokémon.
pub(crate) fn heavy_baton_resume(g: &mut Game, f: crate::cards::CardFrame, results: &[crate::prompts::Res]) -> R {
    let p = f.a[0] as usize;
    let s = f.a[1] as crate::state::SlotId;
    let transfers: SVec<(CardTarget, CardId), 64> = match results.first() {
        Some(crate::prompts::Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    g.st.players[p].slots[s as usize].marker.remove(crate::markers::intern("HEAVY_BATON_MARKER"));
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], NO_CARD)?;
    }
    Ok(())
}

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
    match source {
        None => {
            if g.st.active_pokemon(p) != Some(me) {
                return Ok(());
            }
        }
        Some(src) => {
            if !g.st.slot(src.p as usize, src.s).cards.contains(me) {
                return Ok(());
            }
        }
    }
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if blocked(g, me, origin, at, None) || guard_ok(g, me, p, &b.unless) {
        return Ok(());
    }
    crate::bail!(b.error)
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
    let hp = crate::engine::check::check_hp(g, owner, t.s)?;
    if damage < hp {
        return Ok(());
    }
    if let Effect::PutDamage { survive_on_ten_hp, .. } = g.e_mut(e) {
        *survive_on_ten_hp = true;
    }
    for (s, _, _) in for_each_pokemon(g, owner, PlayerType::BottomPlayer).iter().copied() {
        if g.st.slot(owner, s).tools.contains(me) {
            move_cards(g, ListRef::Slot(owner as u8, s), ListRef::Discard(owner as u8), &[me], me)?;
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
    //! Take-hold stamps and precedence between locks (RULES.md). No two locks in the pool turn each other
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

    #[test]
    fn stamps_follow_take_hold_and_release() {
        // Iron Thorns ex (opp) is Active; Flutter Mane waits on my Bench.
        let mut g = game(json!({"me": {"reset": true, "active": DURA, "bench": [{"card": FM}]}, "opp": {"reset": true, "active": IT}}));
        let me = g.st.active_player as usize;
        let (fm, it) = (top(&g, me, Some(0)), top(&g, 1 - me, None));
        assert_eq!((stamp(&g, fm), stamp(&g, it)), (0, 1), "Iron Thorns ex holds, Flutter Mane isn't Active");
        // Flutter Mane comes in: it turns Iron Thorns ex's lock off though that took hold first.
        let slot = g.st.players[me].bench.as_slice()[0];
        crate::engine::turn::switch_pokemon_silent(&mut g, me, slot).unwrap();
        assert_eq!((stamp(&g, fm), stamp(&g, it)), (2, 0));
        // Flutter Mane leaves: Iron Thorns ex is released and takes hold now.
        let slot = g.st.players[me].bench.as_slice()[0];
        crate::engine::turn::switch_pokemon_silent(&mut g, me, slot).unwrap();
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
    //! Genesect's ACE Nullifier blocks ACE SPEC cards played from the hand only (A-PC6). No pool card
    //! attaches an ACE SPEC Energy from the deck or discard pile, so this can't be a scenario.
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
        g.run_fx(Effect::AttachEnergy { p: me as u8, card, target }).map(|_| ()).map_err(|e| e.0)
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
        g.run_fx(Effect::AttachEnergy { p: me as u8, card, target }).unwrap();
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
    //! `play_locked` and `ability_off` on the lock cards (the query execution and legality share).
    use super::*;
    use serde_json::json;

    const NAMES: [&str; 19] = [
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
        g.st.players[p].hand.iter().find(|c| g.st.cards[*c as usize].def == def).unwrap()
    }

    fn active(g: &Game, p: usize) -> CardId {
        g.st.active_pokemon(p).unwrap()
    }

    fn bench(g: &Game, p: usize, i: usize) -> CardId {
        g.st.slot_pokemon(p, g.st.players[p].bench.as_slice()[i]).unwrap()
    }

    #[test]
    fn jellicent_ex_locks_items_and_tools_while_active() {
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "hand": ["Potion POR 83", "Sacred Charm PFL 93"]},
            "opp": {"reset": true, "active": ["Frillish WHT 44", "Jellicent ex WHT 45"]}}));
        let me = g.st.active_player as usize;
        let (potion, charm) = (hand(&g, me, "Potion POR 83"), hand(&g, me, "Sacred Charm PFL 93"));
        assert_eq!(play_locked(&mut g, me, potion, LockedAction::PlayItem), Some("BLOCKED_BY_ABILITY"));
        assert_eq!(play_locked(&mut g, me, charm, LockedAction::AttachTool), Some("BLOCKED_BY_ABILITY"));
        assert_eq!(play_locked(&mut g, me, potion, LockedAction::PlaySupporter), None);
        // Its owner is not stopped.
        assert_eq!(play_locked(&mut g, 1 - me, potion, LockedAction::PlayItem), None);
        // On the Bench it doesn't lock.
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "hand": ["Potion POR 83"]},
            "opp": {"reset": true, "active": "Duraludon PRE 69", "bench": [{"card": ["Frillish WHT 44", "Jellicent ex WHT 45"]}]}}));
        let potion = hand(&g, me, "Potion POR 83");
        assert_eq!(play_locked(&mut g, me, potion, LockedAction::PlayItem), None);
    }

    #[test]
    fn jellicent_ex_locked_by_iron_thorns_ex_lets_items_through() {
        let mut g = game(json!({"me": {"reset": true, "active": "Iron Thorns ex PRE 32", "hand": ["Potion POR 83"]},
            "opp": {"reset": true, "active": ["Frillish WHT 44", "Jellicent ex WHT 45"]}}));
        let me = g.st.active_player as usize;
        let potion = hand(&g, me, "Potion POR 83");
        let jellicent = active(&g, 1 - me);
        assert_eq!(ability_off(&g, jellicent), Some(true));
        assert_eq!(play_locked(&mut g, me, potion, LockedAction::PlayItem), None);
        // The same with Flutter Mane Active (mine): Midnight Fluttering turns it off.
        let mut g = game(json!({"me": {"reset": true, "active": "Flutter Mane PRE 43", "hand": ["Potion POR 83"]},
            "opp": {"reset": true, "active": ["Frillish WHT 44", "Jellicent ex WHT 45"]}}));
        let potion = hand(&g, me, "Potion POR 83");
        assert_eq!(ability_off(&g, active(&g, 1 - me)), Some(true));
        assert_eq!(play_locked(&mut g, me, potion, LockedAction::PlayItem), None);
    }

    #[test]
    fn arbok_judges_a_hand_card_by_its_printed_data() {
        // With Watchtower in play (id2147) a [C] Pokémon with an Ability still can't be played.
        let mut g = game(json!({"me": {"reset": true, "active": "Hoothoot PRE 77", "hand": ["Noctowl PRE 78", "Duraludon PRE 69", "Team Rocket's Arbok DRI 113", "Antique Root Fossil SCR 130"], "stadium": "Team Rocket's Watchtower ASC 210"},
            "opp": {"reset": true, "active": ["Team Rocket's Ekans DRI 112", "Team Rocket's Arbok DRI 113"]}}));
        let me = g.st.active_player as usize;
        let (noctowl, dura, arbok) = (hand(&g, me, "Noctowl PRE 78"), hand(&g, me, "Duraludon PRE 69"), hand(&g, me, "Team Rocket's Arbok DRI 113"));
        assert_eq!(ability_off(&g, noctowl), Some(false), "printed data in the hand");
        assert_eq!(play_locked(&mut g, me, noctowl, LockedAction::PlayPokemon), Some("BLOCKED_BY_ABILITY"));
        assert_eq!(play_locked(&mut g, me, noctowl, LockedAction::Evolve), None, "Rare Candy isn't covered (no ruling)");
        let fossil = hand(&g, me, "Antique Root Fossil SCR 130");
        assert_eq!(play_locked(&mut g, me, fossil, LockedAction::PlayPokemon), Some("BLOCKED_BY_ABILITY"), "a Fossil with an Ability is played as a Pokémon");
        assert_eq!(play_locked(&mut g, me, dura, LockedAction::PlayPokemon), None, "no Ability");
        assert_eq!(play_locked(&mut g, me, arbok, LockedAction::PlayPokemon), None, "Team Rocket's Pokémon are exempt");
        // Its owner is not stopped.
        assert_eq!(play_locked(&mut g, 1 - me, noctowl, LockedAction::PlayPokemon), None);
    }

    #[test]
    fn arbok_with_iron_thorns_ex_active_still_blocks_a_rule_box_hand_card() {
        let mut g = game(json!({"me": {"reset": true, "active": "Iron Thorns ex PRE 32", "hand": ["Meowth ex POR 62"]},
            "opp": {"reset": true, "active": ["Team Rocket's Ekans DRI 112", "Team Rocket's Arbok DRI 113"]}}));
        let me = g.st.active_player as usize;
        let meowth = hand(&g, me, "Meowth ex POR 62");
        assert_eq!(play_locked(&mut g, me, meowth, LockedAction::PlayPokemon), Some("BLOCKED_BY_ABILITY"));
        // With Flutter Mane Active (mine) Arbok's Ability is off.
        let mut g = game(json!({"me": {"reset": true, "active": "Flutter Mane PRE 43", "hand": ["Meowth ex POR 62"]},
            "opp": {"reset": true, "active": ["Team Rocket's Ekans DRI 112", "Team Rocket's Arbok DRI 113"]}}));
        let meowth = hand(&g, me, "Meowth ex POR 62");
        assert_eq!(ability_off(&g, active(&g, 1 - me)), Some(true));
        assert_eq!(play_locked(&mut g, me, meowth, LockedAction::PlayPokemon), None);
    }

    #[test]
    fn genesect_locks_ace_spec_while_it_has_a_tool() {
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "hand": ["Enriching Energy SSP 191", "Potion POR 83"]},
            "opp": {"reset": true, "active": "Duraludon PRE 69", "bench": [{"card": "Genesect SFA 40", "tool": "Sacred Charm PFL 93"}]}}));
        let me = g.st.active_player as usize;
        let (charm, potion) = (hand(&g, me, "Enriching Energy SSP 191"), hand(&g, me, "Potion POR 83"));
        let _ = bench(&g, 1 - me, 0);
        assert_eq!(play_locked(&mut g, me, charm, LockedAction::AttachEnergy), Some("BLOCKED_BY_EFFECT"));
        assert_eq!(play_locked(&mut g, me, potion, LockedAction::PlayItem), None, "not an ACE SPEC card");
        // Genesect without a Tool doesn't lock.
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "hand": ["Enriching Energy SSP 191"]},
            "opp": {"reset": true, "active": "Genesect SFA 40"}}));
        let charm = hand(&g, me, "Enriching Energy SSP 191");
        assert_eq!(play_locked(&mut g, me, charm, LockedAction::AttachEnergy), None);
        // Flutter Mane Active (mine) turns Genesect's Ability off.
        let mut g = game(json!({"me": {"reset": true, "active": "Flutter Mane PRE 43", "hand": ["Enriching Energy SSP 191"]},
            "opp": {"reset": true, "active": "Genesect SFA 40", "active_tool": "Sacred Charm PFL 93"}}));
        let charm = hand(&g, me, "Enriching Energy SSP 191");
        assert_eq!(play_locked(&mut g, me, charm, LockedAction::AttachEnergy), None);
    }

    #[test]
    fn antique_fossil_cant_retreat_and_palafin_cant_evolve_into() {
        let mut g = game(json!({"me": {"reset": true, "active": "Antique Root Fossil SCR 130", "bench": [{"card": "Duraludon PRE 69"}]},
            "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let me = g.st.active_player as usize;
        let fossil = active(&g, me);
        assert_eq!(play_locked(&mut g, me, fossil, LockedAction::Retreat), Some("CANNOT_RETREAT"));
        // A Pokémon with no such lock retreats; the Fossil on the Bench isn't "the Active Pokémon".
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "bench": [{"card": "Antique Root Fossil SCR 130"}]},
            "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let dura = active(&g, me);
        assert_eq!(play_locked(&mut g, me, dura, LockedAction::Retreat), None);
        // Palafin ex can't be put into play by evolving, whoever does it; Palafin can.
        let mut g = game(json!({"me": {"reset": true, "active": "Finizen TWM 59", "hand": ["Palafin ex PRE 151", "Palafin TWM 60"]},
            "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let (ex, plain) = (hand(&g, me, "Palafin ex PRE 151"), hand(&g, me, "Palafin TWM 60"));
        assert_eq!(play_locked(&mut g, me, ex, LockedAction::Evolve), Some("CANNOT_EVOLVE"));
        assert_eq!(play_locked(&mut g, 1 - me, ex, LockedAction::Evolve), Some("CANNOT_EVOLVE"));
        assert_eq!(play_locked(&mut g, me, plain, LockedAction::Evolve), None);
        assert_eq!(play_locked(&mut g, me, ex, LockedAction::PlayPokemon), None, "only the evolving is locked");
    }

    #[test]
    fn a_stadium_without_use_text_cant_be_used() {
        let mut g = game(json!({"me": {"reset": true, "active": "Duraludon PRE 69", "stadium": "Team Rocket's Watchtower ASC 210"},
            "opp": {"reset": true, "active": "Duraludon PRE 69"}}));
        let me = g.st.active_player as usize;
        let stadium = g.st.stadium_card().unwrap();
        assert_eq!(play_locked(&mut g, me, stadium, LockedAction::UseStadium), Some("CANNOT_USE_STADIUM"));
        assert_eq!(play_locked(&mut g, 1 - me, stadium, LockedAction::UseStadium), Some("CANNOT_USE_STADIUM"));
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
