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
}

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
    /// Today's behavior kept (planned change B-PC-16, Bouffalant SCR): the
    /// passive applies from any zone, not only while the Pokémon is in play.
    pub anywhere: bool,
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
        anywhere: false,
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
}

impl PreventAttackEffectsSpec {
    pub const DEFAULT: PreventAttackEffectsSpec =
        PreventAttackEffectsSpec { subject: SlotPred::Holder, abilities: false, probe_for_attacker: false, needs_source_pokemon: true };
}

/// What a `Prevent` passive stops.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreventWhat {
    /// Damage counters can't be moved (to other Pokémon).
    CounterMoves,
    /// The opponent's Active Pokémon can't be healed. Today's behavior kept (A-PC8): only `Heal`,
    /// not `HealTarget`.
    HealOppActive,
    /// The opponent's Pokémon in play and their attached cards can't be put into the opponent's hand.
    MoveToHandFromOppPlay,
}

/// A prohibition on the opponent's or everyone's effects (vocabulary P5).
pub struct PreventSpec {
    pub what: PreventWhat,
}
/// What a blocker stops.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockWhat {
    /// The Stadium in play can't be used (it has no use text of its own).
    UseStadium,
    /// This Pokémon can't be put into play by evolving (Palafin ex: only by Zero to Hero).
    /// No Ability-lock probe, as today (I-HD-palafin).
    EvolveIntoThis,
    /// The opponent can't play ACE SPEC cards from their hand (Genesect's Ace Canceller) while
    /// the Pokémon has a Tool attached. Today's behavior kept (A-PC6): from any zone, and the
    /// lock probe is made for the playing player.
    AceSpecOfOpponent,
}

pub struct BlockUseSpec {
    pub what: BlockWhat,
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
pub struct SurviveOnTenSpec {}
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
}
pub struct CheckupDamageSpec {}
/// The attacks of the earlier Evolutions in the slot are also this evolved Active Pokémon's
/// (Relicanth's Memory Dive).
pub struct GrantAttacksSpec {}
pub struct AttackFlagsSpec {}
pub struct StatOverrideSpec {}
/// The Pokémon also evolves from `names` (Eevee ex's Rainbow DNA); only a card matching `only`
/// may be played onto it.
pub struct EvolveFromSpec {
    pub names: &'static [&'static str],
    pub only: Pred,
}
pub struct AllowEvolveSpec {}
/// "Can't be affected by Special Conditions" (vocabulary P20).
pub struct ConditionImmunitySpec {
    /// The conditions (empty: all of them).
    pub conds: &'static [SpecialCondition],
    pub subject: SlotPred,
    /// The conditions are removed from effects that would add them.
    pub prevent: bool,
    /// Cleared whenever the table state is checked (today's Festival Grounds, B-PC-38).
    pub sweep: bool,
}
/// The Energy can only be attached to a matching Pokémon (and is discarded from any other).
pub struct AttachGuardSpec {
    pub allow: SlotPred,
}
pub struct BenchSizeSpec {}

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
        Modifier::BlockUse(b) => match b.what {
            BlockWhat::UseStadium => mask(&[k::USE_STADIUM]),
            BlockWhat::EvolveIntoThis => mask(&[k::EVOLVE]),
            BlockWhat::AceSpecOfOpponent => mask(&[k::PLAY_ITEM, k::ATTACH_POKEMON_TOOL, k::ATTACH_ENERGY, k::PLAY_STADIUM]),
        },
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
            PreventWhat::HealOppActive => mask(&[k::HEAL]),
            PreventWhat::MoveToHandFromOppPlay => mask(&[k::MOVE_CARDS]),
        },
        Modifier::PreventAttackEffects(_) => HIDE_N_SNEAK_MASK,
        Modifier::PrizeAdjust(_) => mask(&[k::KNOCK_OUT]),
        Modifier::GrantAttacks(_) => mask(&[k::CHECK_POKEMON_ATTACKS]),
        Modifier::EvolveFrom(_) => mask(&[k::CHECK_TABLE_STATE, k::PLAY_POKEMON]),
        Modifier::AttackCost(_) => mask(&[k::CHECK_ATTACK_COST]),
        Modifier::RetreatCost(_) => mask(&[k::CHECK_RETREAT_COST]),
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
struct Located {
    /// The player whose lock probe applies.
    owner: usize,
    /// The Pokémon the card is part of or attached to.
    held: Option<SlotRef>,
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

fn locate(g: &Game, me: CardId, origin: RuleSource) -> Option<Located> {
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
fn blocked(g: &mut Game, me: CardId, origin: RuleSource, at: Located, affected: Option<SlotRef>) -> bool {
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
        Modifier::DamageDealt(d) => damage_dealt(g, me, e, ps.origin, d),
        Modifier::DamageTaken(d) => damage_taken(g, me, e, ps.origin, d),
        Modifier::PreventDamage(d) => prevent_damage(g, me, e, ps.origin, d),
        Modifier::BlockUse(b) => match (b.what, *g.e(e)) {
            (BlockWhat::UseStadium, Effect::UseStadium { .. }) if g.st.stadium_card() == Some(me) => crate::bail!("CANNOT_USE_STADIUM"),
            (BlockWhat::EvolveIntoThis, Effect::Evolve { card, .. }) if card == me => crate::bail!("CANNOT_EVOLVE"),
            (BlockWhat::AceSpecOfOpponent, _) => ace_spec_of_opponent(g, me, e),
            _ => Ok(()),
        },
        Modifier::ProvidesEnergy(pe) => provides_energy(g, me, e, pe),
        Modifier::ProvidesEnergyBoost(b) => provides_energy_boost(g, me, e, ps.origin, b),
        Modifier::AttachGuard(a) => attach_guard(g, me, e, a),
        Modifier::AttackCost(c) => attack_cost(g, me, e, ps.origin, c),
        Modifier::ConditionImmunity(c) => condition_immunity(g, me, e, ps.origin, c),
        Modifier::AbilityLock(l) => ability_lock(g, me, e, l),
        Modifier::Prevent(p) => prevent(g, me, e, ps.origin, p),
        Modifier::PreventAttackEffects(d) => prevent_attack_effects(g, me, e, ps.origin, d),
        Modifier::PrizeAdjust(d) => prize_adjust(g, me, e, ps.origin, d),
        Modifier::GrantAttacks(_) => grant_attacks(g, me, e, ps.origin),
        Modifier::EvolveFrom(d) => evolve_from(g, me, e, d),
        Modifier::RetreatCost(c) => retreat_cost(g, me, e, ps.origin, c),
        _ => unimplemented!("spec passive not implemented yet (passive.rs)"),
    }
}

fn hp_mod(g: &mut Game, me: CardId, e: EffId, origin: RuleSource, n: i32, subject: &SlotPred, guard: &Cond) -> R {
    let (target, card) = match *g.e(e) {
        Effect::CheckHp { target, card, .. } => (target, card),
        _ => return Ok(()),
    };
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if !slot_pred_m(g, me, target, subject)? || blocked(g, me, origin, at, Some(target)) || !guard_ok(g, me, at.owner, guard) {
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
    match g.e_mut(e) {
        Effect::DealDamage { damage, .. } | Effect::Attack { damage, .. } => *damage += d.amount,
        _ => {}
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
    anywhere: bool,
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
    let located = if anywhere { locate(g, me, RuleSource::CardRule) } else { locate(g, me, origin) };
    let Some(at) = located else { return Ok(None) };
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
    let Some((b, _)) = damage_taken_prelude(g, me, e, origin, d.side, &d.subject, &d.source, &d.guard, d.anywhere, d.from_any_attack)? else { return Ok(()) };
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
        if let Effect::PutDamage { b, .. } = *g.e(e) {
            if origin != RuleSource::CardRule {
                // Today's behavior kept (planned change I-PC1, Cornerstone Mask Ogerpon ex): the
                // Tera rule is skipped when the Ability's own gates are: damage from the owner's
                // own Pokémon, outside the attack phase, or a blocked Ability.
                let Some(at) = locate(g, me, origin) else { return Ok(()) };
                if b.source.p == b.target.p || !is_attack_phase(g) || g.st.slot_pokemon(b.source.p as usize, b.source.s).is_none() || blocked(g, me, origin, at, Some(b.target)) {
                    return Ok(());
                }
            }
            tera_rule(g, e, me);
        }
        return Ok(());
    }
    if d.how == PreventHow::Prevent && !matches!(*g.e(e), Effect::PutDamage { .. }) {
        return Ok(());
    }
    let Some((..)) = damage_taken_prelude(g, me, e, origin, d.side, &d.subject, &d.source, &d.guard, false, false)? else { return Ok(()) };
    match d.how {
        PreventHow::Prevent => g.set_prevent(e, true),
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
    if c.side == Side::Owner && at.owner != p {
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
    if let Effect::CheckAttackCost { cost, ignore_colorless, reduction, .. } = g.e_mut(e) {
        match &c.change {
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
    if c.side == Side::Owner && at.owner != p {
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
    let owner = match l.locker {
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
    };
    let Some(owner) = owner else { return Ok(false) };
    let slot = match g.st.locate(card) {
        None => return Ok(pred(g, card, &l.missing)),
        Some(ListRef::Slot(q, s)) => SlotRef::new(q as usize, s),
        Some(_) => return Ok(false),
    };
    if !pred(g, card, &l.card) {
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

/// The handler of a card that has Damp (also called by Golduck's own handler).
pub fn reduce_damp(g: &mut Game, me: CardId, e: EffId) -> R {
    apply(g, me, e, &Passive { origin: RuleSource::Ability, modifier: Modifier::AbilityLock(DAMP) })
}

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
        _ => {}
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Attack effects

/// Every attack-effect kind (`AtkBase` effects) except the damage steps, plus the counters
/// placed by Abilities: the effect kinds Hide 'n' Sneak and Mist Energy prevent.
pub const HIDE_N_SNEAK_KINDS: [u32; 36] = [
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
        if !slot_pred_m(g, me, t, &d.subject)? {
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
        if matches!(*g.e(e), Effect::ApplyWeakness { .. } | Effect::PutDamage { .. } | Effect::DealDamage { .. }) {
            return Ok(());
        }
        if d.needs_source_pokemon && g.st.slot_pokemon(b.source.p as usize, b.source.s).is_none() {
            return Ok(());
        }
        g.set_prevent(e, true);
        return Ok(());
    }
    if d.abilities {
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
};

/// The handler of a Pokémon with Hide 'n' Sneak (Shuppet, Banette, Poltchageist, ...).
pub fn reduce_hide_n_sneak(g: &mut Game, me: CardId, e: EffId) -> R {
    apply(g, me, e, &Passive { origin: RuleSource::Ability, modifier: Modifier::PreventAttackEffects(HIDE_N_SNEAK) })
}

/// `countHideNSneakPokemonInDiscard(player)`.
pub fn count_hide_n_sneak_in_discard(g: &Game, p: usize) -> usize {
    g.st.players[p]
        .discard
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_pokemon() && d.powers.iter().any(|pw| pw.power_type == PowerType::Ability as u8 && pw.name == "Hide 'n' Sneak")
        })
        .count()
}

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
            Some((a, src)) if src == me && crate::engine::attack::attack_def(g, a).tl_name == name => {}
            _ => return Ok(()),
        }
    }
    let Some(at) = locate(g, me, origin) else { return Ok(()) };
    if blocked(g, me, origin, at, Some(target)) {
        return Ok(());
    }
    if d.by_attack_damage && g.knocked_out_by_attack_damage(p, target).is_none() {
        return Ok(());
    }
    if !guard_ok(g, me, at.owner, &d.guard) {
        return Ok(());
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

/// Genesect's Ace Canceller.
fn ace_spec_of_opponent(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, card) = match *g.e(e) {
        Effect::PlayItem { p, card, .. } | Effect::AttachPokemonTool { p, card, .. } | Effect::AttachEnergy { p, card, .. } | Effect::PlayStadium { p, card } => (p as usize, card),
        _ => return Ok(()),
    };
    if !g.st.cdef(card).has_tag(tag::ACE_SPEC) {
        return Ok(());
    }
    let o = 1 - p;
    let active = for_each_pokemon(g, o, PlayerType::TopPlayer).iter().any(|(s, c, _)| *c == me && !g.st.slot(o, *s).tools.is_empty());
    if !active {
        return Ok(());
    }
    // The lock probe is made for the playing player (today's behavior).
    if is_ability_blocked(g, p, me, None) {
        return Ok(());
    }
    crate::bail!("BLOCKED_BY_EFFECT")
}
