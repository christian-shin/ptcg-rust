//! Attacks and abilities: the `useAttack` / `usePower` generators from
//! `game-effect.ts` and the attack sub-effect reducer (`attack-effect.ts`).

use crate::effects::*;
use crate::energy;
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtkStage {
    AfterConfusion,
    AfterAttackEffect,
    AfterAnimation,
    AfterDealDamage,
    /// The Energy removals that waited for the damage have run (and their prompts resolved).
    AfterDamageEffects,
    AfterAfterAttack,
    /// AfterAttackTriggersEffect resolved (and its prompts).
    AfterTriggers,
    /// Barrage: after the first / second `checkState` wait, and the confirm.
    AfterBarrageCheck1,
    AfterBarrageCheck2,
    AfterBarrageConfirm,
}

#[derive(Clone, Copy, Debug)]
pub struct AttackFrame {
    pub stage: AtkStage,
    pub p: u8,
    pub attack: AttackRef,
    pub attacking: SlotRef,
    /// The `AttackEffect` (live in the arena for the whole attack).
    pub atk: EffId,
    /// The effect that started this generator (UseAttack or Attack).
    pub origin: EffId,
    /// `UseAttackEffect.delegateFrom`: the copy-attack source card.
    pub delegate_from: Option<CardId>,
}

#[derive(Clone, Copy, Debug)]
pub struct PowerFrame {
    pub p: u8,
    pub power: PowerRef,
    pub card: CardId,
    pub bench_target: Option<SlotRef>,
}

/// Attack-flow coin callbacks (none needed by the core yet).
#[derive(Clone, Copy, Debug)]
pub enum AttackCoinCb {
    None,
}

pub fn coin_cb(_g: &mut Game, _a: AttackCoinCb, _result: bool) -> R {
    Ok(())
}

pub fn attack_def(g: &Game, a: AttackRef) -> &'static crate::carddb::AttackDef {
    &g.st.cdef(a.card).attacks[a.idx()]
}

/// Check only, first half: first turn, Special Conditions, which Pokémon
/// attacks (a Benched one for a use-on-Bench attack) and the flags that block
/// it. Returns the attacking slot. Same order as `start_use_attack` always had.
///
/// `granted_first_turn`: an Ability that lets the attack be used on the first turn (Meloetta ex) will write
/// the flag when the `UseAttack` effect reaches it (execution passes `false`: the flag is written by then);
/// legality asks `passive::grants_first_turn_attack` for it.
pub fn can_attack_pre(g: &Game, p: usize, attack: AttackRef, ignore_status: bool, granted_first_turn: bool) -> R<SlotRef> {
    let ad = attack_def(g, attack);
    // `attack.canUseOnFirstTurn` (printed, or written at runtime by Meloetta ex).
    let first_turn_ok = granted_first_turn || g.st.cards[attack.card as usize].attack_first_turn & (1u8 << attack.idx()) != 0;
    if g.st.turn == 1 && !ad.can_use_on_first_turn && !first_turn_ok && !g.st.rules.attack_first_turn {
        crate::bail!("CANNOT_ATTACK_ON_FIRST_TURN");
    }
    let active = g.st.players[p].active;
    let sp = g.st.slot(p, active).special_conditions;
    if (sp.contains(&(SpecialCondition::Paralyzed as u8)) || sp.contains(&(SpecialCondition::Asleep as u8))) && !ignore_status {
        crate::bail!("BLOCKED_BY_SPECIAL_CONDITION");
    }
    let mut attacking = SlotRef::new(p, active);
    for &b in g.st.players[p].bench.iter() {
        if let Some(c) = g.st.slot_pokemon(p, b) {
            if g.st.cdef(c).attacks.iter().any(|a| a.name == ad.name && a.use_on_bench) {
                attacking = SlotRef::new(p, b);
            }
        }
    }
    if g.st.slot(p, attacking.s).cannot_attack_next_turn {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    if g.st.players[p].cannot_attack_turns_remaining > 0 {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    Ok(attacking)
}

/// Is there a "can't attack with this much Energy" effect on the player (so the Energy count is read)?
pub fn attack_max_energy_applies(g: &Game, p: usize) -> bool {
    g.st.players[p].cannot_attack_max_energy_turns_remaining > 0 && g.st.players[p].cannot_attack_max_energy.is_some()
}

/// The Energy count the attacker provides, from the `CheckProvidedEnergy` read.
pub fn max_energy_count(map: &EnergyMap) -> i32 {
    map.iter().map(|m| m.provides.len() as i32).sum()
}

/// The checked read between the halves: with a "can't attack with this much
/// Energy" effect on the player, the Energy count the attacker provides.
pub fn attack_read_max_energy(g: &mut Game, p: usize, attacking: SlotRef) -> R<Option<i32>> {
    if attack_max_energy_applies(g, p) {
        let map = provided_energy_read(g, p, attacking)?;
        return Ok(Some(max_energy_count(&map)));
    }
    Ok(None)
}

/// Check only, second half: the max-Energy rule (given the read) and the
/// blocked attack names.
pub fn can_attack_post(g: &Game, p: usize, attack: AttackRef, attacking: SlotRef, max_energy_count: Option<i32>) -> R {
    let ad = attack_def(g, attack);
    if let (Some(count), Some(max)) = (max_energy_count, g.st.players[p].cannot_attack_max_energy) {
        if count <= max {
            crate::bail!("BLOCKED_BY_EFFECT");
        }
    }
    if g.st.slot(p, attacking.s).cannot_use_attacks_next_turn.contains(&ad.name) {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    if g.st.slot(p, attacking.s).blocked_attack_name_next_turn == Some(ad.name) {
        crate::bail!("BLOCKED_BY_EFFECT");
    }
    if g.st.slot(p, attacking.s).blocked_attack_name_until_leaves_active == Some(ad.name) {
        crate::bail!("CANNOT_USE_ATTACK");
    }
    // cannotAttackMaxEnergy / other blocked attack names /
    // cannotUseAttackUntilLeavesPlay / cannotUseGXAttacks /
    // coinFlipCancelAttackNextTurn: not modeled.
    Ok(())
}

/// The checked read of an attack's current cost (`CheckAttackCost`).
pub fn attack_cost_read(g: &mut Game, p: usize, attack: AttackRef) -> R<Cost> {
    let ad = attack_def(g, attack);
    let mut cost: Cost = SVec::new();
    for &c in ad.cost {
        cost.push(c);
    }
    let (ce, _) = g.run_fx(Effect::CheckAttackCost { p: p as u8, attack, cost, set_cost: None, ignore_colorless: false, reduction: 0, any_reduction: false })?;
    Ok(match ce {
        Effect::CheckAttackCost { cost, .. } => cost,
        _ => SVec::new(),
    })
}

/// The checked read of the Energy a Pokémon provides (`CheckProvidedEnergy`): an attack's and a retreat's payment.
pub fn provided_energy_read(g: &mut Game, p: usize, source: SlotRef) -> R<EnergyMap> {
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source, energy_map: SVec::new() })?;
    Ok(match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
        _ => SVec::new(),
    })
}

/// The cost reads of an attack (`CheckAttackCost`, then `CheckProvidedEnergy`
/// for the attacker) and whether the Energy covers the cost.
pub fn attack_payable(g: &mut Game, p: usize, attack: AttackRef, attacking: SlotRef) -> R<bool> {
    let cost = attack_cost_read(g, p, attack)?;
    let emap = provided_energy_read(g, p, attacking)?;
    Ok(energy::check_enough_energy(emap.as_slice(), cost.as_slice()))
}

pub fn start_use_attack(g: &mut Game, id: EffId) -> R {
    let (p, attack, ignore_status, delegate_from) = match *g.e(id) {
        Effect::UseAttack { p, attack, ignore_status_conditions, delegate_from, .. } => (p as usize, attack, ignore_status_conditions, delegate_from),
        _ => return Ok(()),
    };
    let active = g.st.players[p].active;
    let sp = g.st.slot(p, active).special_conditions;
    let attacking = can_attack_pre(g, p, attack, ignore_status, false)?;
    let max_energy_count = attack_read_max_energy(g, p, attacking)?;
    can_attack_post(g, p, attack, attacking, max_energy_count)?;
    if !attack_payable(g, p, attack, attacking)? {
        crate::bail!("NOT_ENOUGH_ENERGY");
    }
    g.retain_fx(id);
    let f = AttackFrame { stage: AtkStage::AfterConfusion, p: p as u8, attack, attacking, atk: 0, origin: id, delegate_from };
    if sp.contains(&(SpecialCondition::Confused as u8)) {
        let pid = g.player_id(p);
        g.prompt(pid, "FLIP_CONFUSION", PromptKind::CoinFlip, Cont::UseAttack(f));
        return Ok(());
    }
    begin_attack(g, f)
}

fn begin_attack(g: &mut Game, mut f: AttackFrame) -> R {
    let p = f.p as usize;
    g.st.phase = GamePhase::Attack;
    let ad = attack_def(g, f.attack);
    let atk = g.new_fx(Effect::Attack {
        p: f.p,
        opp: (1 - p) as u8,
        attack: f.attack,
        damage: ad.damage,
        ignore_weakness: false,
        ignore_resistance: false,
        ignore_defender_effects: false,
        source: f.attacking,
        barrage_used: false,
    });
    // whileActiveAttackDamageBonus: not modeled.
    f.atk = atk;
    let copycat = g.st.slot_pokemon(p, f.attacking.s);
    if let (Some(src), Some(copycat)) = (f.delegate_from, copycat) {
        // runDelegatedCopiedAttackGenerator: the AttackEffect above is never
        // reduced; the copied attack runs as a clone, then the animation.
        f.stage = AtkStage::AfterAnimation;
        return crate::copy_attack::run_delegated_from_use_attack(g, f, copycat, src);
    }
    g.open_after_damage(atk);
    g.reduce_effect(atk)?;
    if g.has_prompts() {
        f.stage = AtkStage::AfterAttackEffect;
        g.wait_prompt(Cont::UseAttack(f));
        return Ok(());
    }
    animation(g, f)
}

pub fn animation(g: &mut Game, mut f: AttackFrame) -> R {
    f.stage = AtkStage::AfterAnimation;
    let pid = g.player_id(f.p as usize);
    g.prompt(pid, "", PromptKind::Wait, Cont::UseAttack(f));
    Ok(())
}

fn atk_base(g: &Game, atk: EffId) -> AtkBase {
    match *g.e(atk) {
        Effect::Attack { p, opp, attack, source, .. } => {
            let target = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
            AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target, cause: crate::cause::Cause::of_attack_at(g, p, attack, source) }
        }
        _ => panic!("atk_base on non-attack effect"),
    }
}

fn deal_damage(g: &mut Game, mut f: AttackFrame) -> R {
    let (p, opp, attack) = match *g.e(f.atk) {
        Effect::Attack { p, opp, attack, .. } => (p, opp, attack),
        _ => unreachable!(),
    };
    g.run_fx_unit(Effect::BeforeDoingDamage { attack_effect: f.atk, p, opp, attack })?;
    let damage = match *g.e(f.atk) {
        Effect::Attack { damage, .. } => damage,
        _ => 0,
    };
    if damage > 0 {
        let b = atk_base(g, f.atk);
        crate::engine::damage::deal(g, b, damage, true)?;
        if g.has_prompts() {
            f.stage = AtkStage::AfterDealDamage;
            g.wait_prompt(Cont::UseAttack(f));
            return Ok(());
        }
    }
    after_attack(g, f)
}

/// `RUN_AFTER_DAMAGE_EFFECTS`: Energy removed as an effect of the attack leaves after the damage.
fn after_attack(g: &mut Game, mut f: AttackFrame) -> R {
    g.run_after_damage(f.atk)?;
    if g.has_prompts() {
        f.stage = AtkStage::AfterDamageEffects;
        g.wait_prompt(Cont::UseAttack(f));
        return Ok(());
    }
    after_attack_effect(g, f)
}

fn after_attack_effect(g: &mut Game, mut f: AttackFrame) -> R {
    let p = f.p as usize;
    // Tenacious Body / Durable Body: the coin is flipped after all the damage is done (ruling 1770).
    crate::prefabs::resolve_survive_coin_flips(g)?;
    g.run_fx_unit(Effect::AfterAttack { p: f.p, opp: (1 - p) as u8, attack: f.attack, atk: f.atk })?;
    if g.has_prompts() {
        f.stage = AtkStage::AfterAfterAttack;
        g.wait_prompt(Cont::UseAttack(f));
        return Ok(());
    }
    attack_triggers(g, f)
}

/// Effects that trigger on the Defending Pokémon (Handheld Fan) resolve once everything the attack
/// did, prompts included, is over (rulings 1625, 1650, 1651).
fn attack_triggers(g: &mut Game, f: AttackFrame) -> R {
    let p = f.p as usize;
    // Damage done by the attack's after-damage effects (spec cards: Bench
    // damage after the damage step) gets its Tenacious Body coin here, once
    // all of the attack's damage is done (ruling 1770).
    crate::prefabs::resolve_survive_coin_flips(g)?;
    g.run_fx_unit(Effect::AfterAttackTriggers { p: f.p, opp: (1 - p) as u8, attack: f.attack })?;
    attack_triggers_loop(g, f)
}

/// Step 7 (Advanced Player's Rulebook E-03): one trigger at a time; a trigger that opens a prompt is answered before
/// the next one resolves.
fn attack_triggers_loop(g: &mut Game, mut f: AttackFrame) -> R {
    while g.attack_triggers_pending(f.atk) {
        g.resolve_next_attack_trigger(f.atk)?;
        if g.has_prompts() {
            f.stage = AtkStage::AfterTriggers;
            g.wait_prompt(Cont::UseAttack(f));
            return Ok(());
        }
    }
    g.close_attack_triggers(f.atk);
    if g.has_prompts() {
        f.stage = AtkStage::AfterTriggers;
        g.wait_prompt(Cont::UseAttack(f));
        return Ok(());
    }
    finish_attack(g, f)
}

/// `attack.barrage`: the printed flag or a runtime write by the card
/// (`this.attacks[i].barrage = ...`, e.g. Festival Lead).
pub fn attack_barrage(g: &Game, a: AttackRef) -> bool {
    if a.is_clone() {
        // The clone's own flag (`cloneAttacks` copy, plus delegated writes).
        let bit = 1u8 << a.idx();
        let serial = (a.index >> 4) & 7;
        return attack_def(g, a).barrage
            || g.copy_sessions.as_slice().iter().any(|s| s.source == a.card && s.serial & 7 == serial && s.barrage & bit != 0);
    }
    attack_def(g, a).barrage || g.st.cards[a.card as usize].attack_barrage & (1u8 << a.idx()) != 0
}

fn finish_attack(g: &mut Game, f: AttackFrame) -> R {
    let barrage_used = match *g.e(f.origin) {
        Effect::UseAttack { barrage_used, .. } | Effect::Attack { barrage_used, .. } => barrage_used,
        _ => false,
    };
    // hasBarragePower (power.barrage): no pool card sets it.
    if attack_barrage(g, f.attack) && !barrage_used {
        return barrage_check(g, f, AtkStage::AfterBarrageCheck1);
    }
    g.release_fx(f.atk);
    g.release_fx(f.origin);
    g.run_fx_unit(Effect::EndTurn { p: f.p })?;
    Ok(())
}

/// `state = checkState(store, state); if (store.hasPrompts()) yield waitPrompt`
/// (twice), then `ConfirmPrompt(WANT_TO_USE_ABILITY)`.
fn barrage_check(g: &mut Game, mut f: AttackFrame, stage: AtkStage) -> R {
    let below = g.waits.len();
    crate::engine::knockout::state_check(g, crate::game::OnComplete::None)?;
    if g.has_prompts() {
        f.stage = stage;
        // The attack resumes after the check's own continuations (its remaining prize and new-Active
        // prompts, then the next check), not before them. Twinleaf pops the waits last-in first-out, so the
        // attack continued while the check was half done: the Prize for the Defending Pokemon went unasked
        // and, once the turn ended, a stale prompt asked for a second new Active (ILLEGAL_ACTION).
        g.waits.insert(below, Cont::UseAttack(f));
        return Ok(());
    }
    barrage_after_check(g, f, stage)
}

fn barrage_after_check(g: &mut Game, mut f: AttackFrame, stage: AtkStage) -> R {
    if stage == AtkStage::AfterBarrageCheck1 {
        return barrage_check(g, f, AtkStage::AfterBarrageCheck2);
    }
    // The second use runs through `useAttack` again: it isn't offered when it
    // would throw (the attacker lost its Energy, e.g. to Handheld Fan, or is now
    // Asleep / Paralyzed); the turn just ends.
    if attack_barrage(g, f.attack) && !barrage_can_attack_again(g, &f)? {
        g.release_fx(f.atk);
        g.release_fx(f.origin);
        g.run_fx_unit(Effect::EndTurn { p: f.p })?;
        return Ok(());
    }
    f.stage = AtkStage::AfterBarrageConfirm;
    let pid = g.player_id(f.p as usize);
    g.prompt(pid, "WANT_TO_USE_ABILITY", PromptKind::Confirm, Cont::UseAttack(f));
    Ok(())
}

/// The status and Energy checks `useAttack` would make for the barrage attack.
fn barrage_can_attack_again(g: &mut Game, f: &AttackFrame) -> R<bool> {
    let p = f.p as usize;
    let active = g.st.players[p].active;
    let sp = g.st.slot(p, active).special_conditions;
    if sp.contains(&(SpecialCondition::Paralyzed as u8)) || sp.contains(&(SpecialCondition::Asleep as u8)) {
        return Ok(false);
    }
    let ad = attack_def(g, f.attack);
    let mut attacking = SlotRef::new(p, active);
    for &b in g.st.players[p].bench.iter() {
        if let Some(c) = g.st.slot_pokemon(p, b) {
            if g.st.cdef(c).attacks.iter().any(|a| a.name == ad.name && a.use_on_bench) {
                attacking = SlotRef::new(p, b);
            }
        }
    }
    let mut cost: Cost = SVec::new();
    for &c in ad.cost {
        cost.push(c);
    }
    let (ce, _) = g.run_fx(Effect::CheckAttackCost { p: f.p, attack: f.attack, cost, set_cost: None, ignore_colorless: false, reduction: 0, any_reduction: false })?;
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: f.p, source: attacking, energy_map: SVec::new() })?;
    let cost = match ce {
        Effect::CheckAttackCost { cost, .. } => cost,
        _ => SVec::new(),
    };
    let emap = match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
        _ => SVec::new(),
    };
    Ok(energy::check_enough_energy(emap.as_slice(), cost.as_slice()))
}

/// Confirm answered: attack again with a `UseAttackEffect` marked
/// `_barrageUsed`, run straight through `useAttack` (never reduced, so no
/// card sees it), or end the turn.
fn barrage_confirm(g: &mut Game, f: AttackFrame, want: bool) -> R {
    g.release_fx(f.atk);
    g.release_fx(f.origin);
    if !want {
        g.run_fx_unit(Effect::EndTurn { p: f.p })?;
        return Ok(());
    }
    let p = f.p as usize;
    let source = SlotRef::new(p, g.st.players[p].active);
    let id = g.new_fx(Effect::UseAttack { p: f.p, attack: f.attack, source, ignore_status_conditions: false, barrage_used: true, delegate_from: None });
    let r = start_use_attack(g, id);
    g.release_fx(id);
    r
}

pub fn resume_use_attack(g: &mut Game, f: AttackFrame, res: Res) -> R {
    match f.stage {
        AtkStage::AfterConfusion => {
            let heads = res.as_bool();
            // The flip of a Confused Pokémon trying to attack.
            crate::engine::condition::coin_flipped(g, f.p as usize, crate::spec::event::CoinPurpose::Confusion, heads, crate::engine::condition::by_condition(f.p as usize))?;
            if !heads {
                let p = f.p as usize;
                let a = g.st.players[p].active;
                let conf = g.st.slot(p, a).confusion_damage;
                // Confusion's rule puts its damage counters on the Pokémon (a PlaceCounters by the Special Condition).
                crate::engine::damage::place(g, SlotRef::new(p, a), conf, crate::engine::condition::by_condition(p))?;
                // A failed attack attempt while Confused isn't an attack used
                // (phase 4b, ruling n=1621): the playerLastAttack stamp is voided.
                g.st.player_last_attack_turn[p] = -1;
                g.release_fx(f.origin);
                g.run_fx_unit(Effect::EndTurn { p: f.p })?;
                return Ok(());
            }
            begin_attack(g, f)
        }
        AtkStage::AfterAttackEffect => animation(g, f),
        // Delegated copies already dealt damage and ran AfterAttackEffect.
        AtkStage::AfterAnimation if f.delegate_from.is_some() => finish_attack(g, f),
        AtkStage::AfterAnimation => deal_damage(g, f),
        AtkStage::AfterDealDamage => after_attack(g, f),
        AtkStage::AfterDamageEffects => after_attack_effect(g, f),
        AtkStage::AfterAfterAttack => attack_triggers(g, f),
        AtkStage::AfterTriggers => attack_triggers_loop(g, f),
        AtkStage::AfterBarrageCheck1 | AtkStage::AfterBarrageCheck2 => barrage_after_check(g, f, f.stage),
        AtkStage::AfterBarrageConfirm => barrage_confirm(g, f, res.as_bool()),
    }
}

/// The lock probe made before an Ability is used: a stand-in `Power` effect every card sees; a lock refuses it.
pub fn power_use_blocked(g: &mut Game, p: usize, power: PowerRef, card: CardId) -> bool {
    g.run_fx(Effect::Power { p: p as u8, power, card, target: None, probe: true }).is_err()
}

pub fn start_use_power(g: &mut Game, id: EffId) -> R {
    let (p, power, card, target, bench_target) = match *g.e(id) {
        Effect::UsePower { p, power, card, target, bench_target } => (p as usize, power, card, target, bench_target),
        _ => return Ok(()),
    };
    // assertActivatedPowerNotLocked: probe with a stand-in power.
    if power_use_blocked(g, p, power, card) {
        crate::bail!("CANNOT_USE_POWER");
    }
    let f = PowerFrame { p: p as u8, power, card, bench_target };
    if target.slot == SlotType::Active || target.slot == SlotType::Bench {
        let pid = g.player_id(p);
        g.prompt(pid, "", PromptKind::Wait, Cont::UsePower(f));
        return Ok(());
    }
    resume_use_power(g, f)
}

pub fn resume_use_power(g: &mut Game, f: PowerFrame) -> R {
    g.run_fx_unit(Effect::Power { p: f.p, power: f.power, card: f.card, target: f.bench_target, probe: false })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// attackReducer

pub fn reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::AttackTrigger { opp, card, target, source, source_in_play, retaliate: Some(r), .. } => {
            // Resolution of a revenge trap: an EffectOfAttack attributed to the retaliator so Mist Energy blocks it.
            // The Attacking Pokémon must still be in play (ruling 530) and takes the counters wherever it is (rulings
            // 482, 1839); the trap is an effect of the damaged Pokémon, gone when that Pokémon left play.
            if card == r.source_card && r.damage > 0 && source_in_play && g.st.slot(target.p as usize, target.s).cards.contains(r.source_card) {
                // The retaliation is an effect of the retaliator's own earlier attack (id2408, id1958): its counters on the
                // Attacking Pokémon are a PlaceCounters by that attack.
                let cause = crate::cause::Cause::attack(opp, Some(r.source_card), r.attack);
                crate::engine::damage::place(g, source, r.damage, cause)?;
            }
            Ok(())
        }
        Effect::MoveOpponentEnergy { b, card, destination } => {
            // The attack's effect on the Pokémon wasn't prevented: the card moves (MoveEnergy / MoveTool).
            crate::engine::attach::move_attached(g, card, b.target, destination, b.cause)?;
            Ok(())
        }
        _ => Ok(()),
    }
}
