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
use crate::state::ListRef;
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

pub struct PreventAttackEffectsSpec {}
pub struct PreventSpec {}
/// What a blocker stops.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockWhat {
    /// The Stadium in play can't be used (it has no use text of its own).
    UseStadium,
    /// This Pokémon can't be put into play by evolving (Palafin ex: only by Zero to Hero).
    /// No Ability-lock probe, as today (I-HD-palafin).
    EvolveIntoThis,
}

pub struct BlockUseSpec {
    pub what: BlockWhat,
}
pub struct AbilityLockSpec {}
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
pub struct PrizeAdjustSpec {}
pub struct CheckupDamageSpec {}
pub struct GrantAttacksSpec {}
pub struct AttackFlagsSpec {}
pub struct StatOverrideSpec {}
pub struct EvolveFromSpec {}
pub struct AllowEvolveSpec {}
pub struct ConditionImmunitySpec {}
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
        },
        Modifier::ProvidesEnergy(_) | Modifier::ProvidesEnergyBoost(_) => mask(&[k::CHECK_PROVIDED_ENERGY]),
        Modifier::AttachGuard(_) => mask(&[k::ATTACH_ENERGY, k::CHECK_TABLE_STATE]),
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
            _ => Ok(()),
        },
        Modifier::ProvidesEnergy(pe) => provides_energy(g, me, e, pe),
        Modifier::ProvidesEnergyBoost(b) => provides_energy_boost(g, me, e, ps.origin, b),
        Modifier::AttachGuard(a) => attach_guard(g, me, e, a),
        Modifier::AttackCost(c) => attack_cost(g, me, e, ps.origin, c),
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
    if !slot_pred(g, me, target, subject)? || blocked(g, me, origin, at, Some(target)) || !guard_ok(g, me, at.owner, guard) {
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
    if !slot_pred(g, me, source, &d.attacker)? || !guard_ok(g, me, at.owner, &d.guard) {
        return Ok(());
    }
    if let Some(t) = target {
        if !slot_pred(g, me, t, &d.target)? {
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
    if !slot_pred(g, me, t, subject)? || blocked(g, me, origin, at, Some(t)) {
        return Ok(None);
    }
    if !guard_ok(g, me, at.owner, guard) || !slot_pred(g, me, b.source, source)? {
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
        if !slot_pred(g, me, source, &entry.when)? {
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
            if !slot_pred(g, me, target, &a.allow)? {
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
                    if g.st.slot_pokemon(p, s).is_some() && !slot_pred(g, me, t, &a.allow)? {
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
    if !slot_pred(g, me, active, &c.subject)? || blocked(g, me, origin, at, Some(active)) {
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
        if slot_pred(g, me, active, &c.subject)? {
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
