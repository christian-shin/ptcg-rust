//! Attacks and abilities: the `useAttack` / `usePower` generators from
//! `game-effect.ts` and the attack sub-effect reducer (`attack-effect.ts`).

use crate::effects::*;
use crate::energy;
use crate::engine::game_effect::stats_effect;
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::markers::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtkStage {
    AfterConfusion,
    AfterAttackEffect,
    AfterAnimation,
    AfterDealDamage,
    AfterAfterAttack,
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

pub fn start_use_attack(g: &mut Game, id: EffId) -> R {
    let (p, attack, ignore_status, delegate_from) = match *g.e(id) {
        Effect::UseAttack { p, attack, ignore_status_conditions, delegate_from, .. } => (p as usize, attack, ignore_status_conditions, delegate_from),
        _ => return Ok(()),
    };
    let ad = attack_def(g, attack);
    // `attack.canUseOnFirstTurn` (printed, or written at runtime by Meloetta ex).
    let first_turn_ok = g.st.cards[attack.card as usize].attack_first_turn & (1u8 << attack.idx()) != 0;
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
    if g.st.players[p].cannot_attack_max_energy_turns_remaining > 0 {
        if let Some(max) = g.st.players[p].cannot_attack_max_energy {
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: attacking, energy_map: SVec::new() })?;
            let count: i32 = match pe {
                Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|m| m.provides.len() as i32).sum(),
                _ => 0,
            };
            if count <= max {
                crate::bail!("BLOCKED_BY_EFFECT");
            }
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

    let mut cost: Cost = SVec::new();
    for &c in ad.cost {
        cost.push(c);
    }
    let (ce, _) = g.run_fx(Effect::CheckAttackCost { p: p as u8, attack, cost, set_cost: None, ignore_colorless: false })?;
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: attacking, energy_map: SVec::new() })?;
    let cost = match ce {
        Effect::CheckAttackCost { cost, .. } => cost,
        _ => SVec::new(),
    };
    let emap = match pe {
        Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
        _ => SVec::new(),
    };
    if !energy::check_enough_energy(emap.as_slice(), cost.as_slice()) {
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
            AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target }
        }
        _ => panic!("atk_base on non-attack effect"),
    }
}

fn deal_damage(g: &mut Game, mut f: AttackFrame) -> R {
    let (p, opp, attack) = match *g.e(f.atk) {
        Effect::Attack { p, opp, attack, .. } => (p, opp, attack),
        _ => unreachable!(),
    };
    g.run_fx(Effect::BeforeDoingDamage { attack_effect: f.atk, p, opp, attack })?;
    let damage = match *g.e(f.atk) {
        Effect::Attack { damage, .. } => damage,
        _ => 0,
    };
    if damage > 0 {
        let b = atk_base(g, f.atk);
        g.run_fx(Effect::DealDamage { b, damage })?;
        if g.has_prompts() {
            f.stage = AtkStage::AfterDealDamage;
            g.wait_prompt(Cont::UseAttack(f));
            return Ok(());
        }
    }
    after_attack(g, f)
}

fn after_attack(g: &mut Game, mut f: AttackFrame) -> R {
    let p = f.p as usize;
    g.run_fx(Effect::AfterAttack { p: f.p, opp: (1 - p) as u8, attack: f.attack })?;
    if g.has_prompts() {
        f.stage = AtkStage::AfterAfterAttack;
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
    g.run_fx(Effect::EndTurn { p: f.p })?;
    Ok(())
}

/// `state = checkState(store, state); if (store.hasPrompts()) yield waitPrompt`
/// (twice), then `ConfirmPrompt(WANT_TO_USE_ABILITY)`.
fn barrage_check(g: &mut Game, mut f: AttackFrame, stage: AtkStage) -> R {
    crate::engine::check::check_state(g, crate::game::OnComplete::None)?;
    if g.has_prompts() {
        f.stage = stage;
        g.wait_prompt(Cont::UseAttack(f));
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
        g.run_fx(Effect::EndTurn { p: f.p })?;
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
    let (ce, _) = g.run_fx(Effect::CheckAttackCost { p: f.p, attack: f.attack, cost, set_cost: None, ignore_colorless: false })?;
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
        g.run_fx(Effect::EndTurn { p: f.p })?;
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
            if !heads {
                let p = f.p as usize;
                let a = g.st.players[p].active;
                let conf = g.st.slot(p, a).confusion_damage;
                g.st.players[p].slots[a as usize].damage += conf;
                // A failed attack attempt while Confused isn't an attack used
                // (phase 4b, ruling n=1621): the playerLastAttack stamp is voided.
                g.st.player_last_attack_turn[p] = -1;
                g.release_fx(f.origin);
                g.run_fx(Effect::EndTurn { p: f.p })?;
                return Ok(());
            }
            begin_attack(g, f)
        }
        AtkStage::AfterAttackEffect => animation(g, f),
        // Delegated copies already dealt damage and ran AfterAttackEffect.
        AtkStage::AfterAnimation if f.delegate_from.is_some() => finish_attack(g, f),
        AtkStage::AfterAnimation => deal_damage(g, f),
        AtkStage::AfterDealDamage => after_attack(g, f),
        AtkStage::AfterAfterAttack => finish_attack(g, f),
        AtkStage::AfterBarrageCheck1 | AtkStage::AfterBarrageCheck2 => barrage_after_check(g, f, f.stage),
        AtkStage::AfterBarrageConfirm => barrage_confirm(g, f, res.as_bool()),
    }
}

pub fn start_use_power(g: &mut Game, id: EffId) -> R {
    let (p, power, card, target, bench_target) = match *g.e(id) {
        Effect::UsePower { p, power, card, target, bench_target } => (p as usize, power, card, target, bench_target),
        _ => return Ok(()),
    };
    // assertActivatedPowerNotLocked: probe with a stand-in power.
    if g.run_fx(Effect::Power { p: p as u8, power, card, target: None, probe: true }).is_err() {
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
    g.run_fx(Effect::Power { p: f.p, power: f.power, card: f.card, target: f.bench_target, probe: false })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// attackReducer

fn apply_put_damage(g: &mut Game, id: EffId) -> R {
    let (b, damage) = match *g.e(id) {
        Effect::PutDamage { b, damage, .. } => (b, damage),
        _ => return Ok(()),
    };
    let t = b.target;
    let target_card = match g.st.slot_pokemon(t.p as usize, t.s) {
        Some(c) => c,
        None => crate::bail!("ILLEGAL_ACTION"),
    };
    let damage = damage.max(0);
    g.st.players[t.p as usize].slots[t.s as usize].damage += damage;
    if damage > 0 {
        g.st.players[t.p as usize].marker.add_to_state(DAMAGE_DEALT_MARKER);
        g.st.cards[target_card as usize].damage_taken_last_turn += damage;
        // `surviveOnTenHPReason` set by a card (e.g. SURVIVE_ON_TEN_IF_FULL_HP):
        // CheckHpEffect(effect.player, target); at or over HP → HP - 10.
        let survive = matches!(*g.e(id), Effect::PutDamage { survive_on_ten_hp: true, .. });
        if survive {
            let (tp, ts) = (t.p as usize, t.s);
            let card = g.st.slot_pokemon(tp, ts);
            if card.is_some() {
                g.st.players[tp].slots[ts as usize].hp_bonus = 0;
            }
            g.run_fx(Effect::CheckHp { p: b.player, target: t, card })?;
            let hp = crate::engine::check::hp_of(g, tp, ts, card);
            if g.st.slot(tp, ts).damage >= hp {
                g.st.players[tp].slots[ts as usize].damage = hp - 10;
            }
        }
        let mut ab = b;
        ab.target = t;
        g.run_fx(Effect::AfterDamage { b: ab, damage })?;
    }
    Ok(())
}

/// `shouldPreventAttackEffects(state, effect)` (empty filter only).
fn should_prevent_attack_effects(g: &Game, id: EffId) -> bool {
    let b = match g.e(id).atk_base() {
        Some(b) => *b,
        None => return false,
    };
    if !g.st.slot(b.target.p as usize, b.target.s).prevent_effects_of_attacks_next_turn {
        return false;
    }
    if b.source.p == b.target.p {
        return false;
    }
    if g.st.slot_pokemon(b.source.p as usize, b.source.s).is_none() {
        return false;
    }
    !matches!(*g.e(id), Effect::ApplyWeakness { .. } | Effect::PutDamage { .. } | Effect::DealDamage { .. })
}

pub fn reducer(g: &mut Game, id: EffId) -> R {
    if should_prevent_attack_effects(g, id) {
        return Ok(());
    }
    match *g.e(id) {
        Effect::PutDamage { b, damage, weakness_applied, .. } => {
            let t = b.target;
            if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
                crate::bail!("ILLEGAL_ACTION");
            }
            let opp = 1 - b.player as usize;
            let mut damage = damage;
            // Defending Pokémon's attacks do N less (before W/R), direct PutDamage only.
            let src_red = g.st.slot(b.source.p as usize, b.source.s).attack_damage_reduction_next_turn;
            if !weakness_applied && src_red > 0 {
                damage = (damage - src_red).max(0);
            }
            if t.p as usize == opp && t.s == g.st.players[opp].active && !weakness_applied {
                let (ig_w, ig_r) = match *g.e(b.attack_effect) {
                    Effect::Attack { ignore_weakness, ignore_resistance, .. } => (ignore_weakness, ignore_resistance),
                    _ => (false, false),
                };
                let mut wb = b;
                wb.target = t;
                let (e, _) = g.run_fx(Effect::ApplyWeakness { b: wb, damage, ignore_weakness: ig_w, ignore_resistance: ig_r })?;
                if let Effect::ApplyWeakness { damage: d, .. } = e {
                    damage = d;
                }
                g.st.players[t.p as usize].marker.add_to_state(DAMAGE_DEALT_MARKER);
            }
            // shouldPreventAttackDamage (sourceStage / sourceCardTypes filters modeled).
            let prevent = g.st.phase == GamePhase::Attack
                && g.st.slot(t.p as usize, t.s).prevent_damage_next_turn
                && match g.st.slot_pokemon(b.source.p as usize, b.source.s) {
                    Some(sc) => {
                        let d = g.st.cdef(sc);
                        g.st.slot(t.p as usize, t.s).prevent_damage_filter.matches(d.stage, d.card_type, d.powers.iter().any(|pw| pw.power_type == PowerType::Ability as u8))
                    }
                    None => false,
                };
            if prevent {
                if let Effect::PutDamage { damage: d, .. } = g.e_mut(id) {
                    *d = damage;
                }
                return Ok(());
            }
            let red = g.st.slot(t.p as usize, t.s).damage_reduction_next_turn;
            if red != 0 {
                damage = (damage - red).max(0);
            }
            // "During your next turn, the Defending Pokémon takes N more damage."
            {
                let ts = g.st.slot(t.p as usize, t.s);
                if ts.defending_extra_damage_next_turn > 0 && !ts.defending_extra_damage_pending && ts.defending_extra_damage_attacker == Some(b.player) {
                    damage += ts.defending_extra_damage_next_turn;
                }
            }
            if let Effect::PutDamage { damage: d, .. } = g.e_mut(id) {
                *d = damage;
            }
            apply_put_damage(g, id)
        }
        Effect::DealDamage { b, damage } => {
            let mut damage = damage;
            let src_red = g.st.slot(b.source.p as usize, b.source.s).attack_damage_reduction_next_turn;
            if src_red > 0 {
                damage = (damage - src_red).max(0);
                if let Effect::DealDamage { damage: d, .. } = g.e_mut(id) {
                    *d = damage;
                }
            }
            let (ig_w, ig_r) = match *g.e(b.attack_effect) {
                Effect::Attack { ignore_weakness, ignore_resistance, .. } => (ignore_weakness, ignore_resistance),
                _ => (false, false),
            };
            let (e, _) = g.run_fx(Effect::ApplyWeakness { b, damage, ignore_weakness: ig_w, ignore_resistance: ig_r })?;
            let d = match e {
                Effect::ApplyWeakness { damage, .. } => damage,
                _ => damage,
            };
            g.run_fx(Effect::PutDamage { b, damage: d, weakness_applied: true, survive_on_ten_hp: false })?;
            Ok(())
        }
        Effect::PutCounters { b, damage } => {
            let t = b.target;
            if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
                crate::bail!("ILLEGAL_ACTION");
            }
            g.st.players[t.p as usize].slots[t.s as usize].damage += damage.max(0);
            Ok(())
        }
        Effect::AfterDamage { b, damage } => {
            g.st.players[b.target.p as usize].marker.add_to_state(DAMAGE_DEALT_MARKER);
            // Revenge trap (getActiveRetaliateOnDamage; `{ damage }` options only).
            let t = b.target;
            let slot = g.st.slot(t.p as usize, t.s);
            let active = if slot.retaliate_on_damage_next_turn_pending.is_some() { None } else { slot.retaliate_on_damage_next_turn };
            if let Some(r) = active {
                if damage > 0 && t.p != b.player && g.st.phase == GamePhase::Attack && r.damage > 0 {
                    let mut src = t;
                    let ap = r.attacker as usize;
                    for s in g.st.players[ap].in_play().iter() {
                        if g.st.slot_pokemon(ap, *s) == Some(r.source_card) {
                            src = SlotRef::new(ap, *s);
                        }
                    }
                    let rb = AtkBase { attack_effect: b.attack_effect, player: t.p, opponent: b.player, attack: r.attack, source: src, target: b.source };
                    g.run_fx(Effect::RetaliateDamage { b: rb, damage: r.damage })?;
                }
            }
            Ok(())
        }
        Effect::KnockOutOpponent { b, .. } => {
            // KnockOutAttackEffect on the target, then TAKE_X_PRIZES.
            let t = b.target;
            if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
                crate::bail!("ILLEGAL_ACTION");
            }
            let (ko, prevented) = g.run_fx(Effect::KnockOut { p: t.p, target: t, prize_count: 1, prize_destination: None, attack: Some(b.attack), defer_removal: false })?;
            if !prevented {
                let pc = match ko {
                    Effect::KnockOut { prize_count, .. } => prize_count,
                    _ => 1,
                };
                if let Effect::KnockOutOpponent { knocked_out, prize_count, .. } = g.e_mut(id) {
                    *knocked_out = true;
                    *prize_count = pc;
                }
                crate::engine::check::take_x_prizes(g, b.player as usize, pc)?;
            }
            Ok(())
        }
        Effect::KnockOutPlayer { b, .. } => {
            // KnockOutAttackEffect on the target (the attacker's side), then
            // TAKE_X_PRIZES for the opponent.
            let t = b.target;
            if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
                crate::bail!("ILLEGAL_ACTION");
            }
            let (ko, prevented) = g.run_fx(Effect::KnockOut { p: t.p, target: t, prize_count: 1, prize_destination: None, attack: Some(b.attack), defer_removal: false })?;
            if !prevented {
                let pc = match ko {
                    Effect::KnockOut { prize_count, .. } => prize_count,
                    _ => 1,
                };
                if let Effect::KnockOutPlayer { knocked_out, prize_count, .. } = g.e_mut(id) {
                    *knocked_out = true;
                    *prize_count = pc;
                }
                crate::engine::check::take_x_prizes(g, b.opponent as usize, pc)?;
            }
            Ok(())
        }
        Effect::DiscardCards { b, cards } => {
            let owner = b.target.p;
            g.move_cards_to(b.target.list(), cards.as_slice(), ListRef::Discard(owner));
            Ok(())
        }
        Effect::CardsToHand { b, cards } => {
            let owner = b.target.p;
            g.move_cards_to(b.target.list(), cards.as_slice(), ListRef::Hand(owner));
            Ok(())
        }
        Effect::GustOpponentBench { b } => {
            crate::engine::turn::switch_pokemon(g, b.opponent as usize, b.target.s)?;
            Ok(())
        }
        Effect::MoveOpponentEnergy { b, card, destination } => {
            // MoveOpponentEnergyEffect: `target.moveCardTo(card, destination)`.
            g.move_card_to(b.target.list(), card, destination.list());
            Ok(())
        }
        Effect::AddMarker { b, marker, marker_source } => {
            g.st.players[b.target.p as usize].slots[b.target.s as usize].marker.add(marker, marker_source, SourceType::None, TargetScope::None);
            Ok(())
        }
        Effect::HealTarget { b, damage } => {
            let owner = b.target.p;
            g.run_fx(Effect::Heal { p: owner, target: b.target, damage })?;
            Ok(())
        }
        Effect::AddSpecialConditions { b, conditions, poison_damage, burn_damage, confusion_damage } => {
            let slot = &mut g.st.players[b.target.p as usize].slots[b.target.s as usize];
            for &c in conditions.iter() {
                crate::engine::phase::add_condition(slot, SpecialCondition::from_u8(c));
            }
            if let Some(v) = poison_damage {
                slot.poison_damage = v;
            }
            if let Some(v) = burn_damage {
                slot.burn_damage = v;
            }
            if let Some(v) = confusion_damage {
                slot.confusion_damage = v;
            }
            Ok(())
        }
        Effect::PlayLock { b, locks, turns_remaining, both_players, attacker_turns_remaining } => {
            let opp = b.opponent as usize;
            crate::engine::phase::apply_play_locks(&mut g.st.players[opp], locks, turns_remaining.unwrap_or(1));
            if both_players {
                let me = b.player as usize;
                crate::engine::phase::apply_play_locks(&mut g.st.players[me], locks, attacker_turns_remaining.unwrap_or(2));
            }
            Ok(())
        }
        Effect::IncreaseAttackCostNextTurn { b } => {
            // applyEffect(): the opponent's current Active.
            let o = b.opponent as usize;
            let a = g.st.players[o].active;
            let slot = &mut g.st.players[o].slots[a as usize];
            slot.attack_cost_increase_next_turn_pending = 1;
            slot.attack_cost_increase_next_turn_attacker = Some(b.player);
            Ok(())
        }
        Effect::IncreaseRetreatCostNextTurn { b } => {
            let o = b.opponent as usize;
            let a = g.st.players[o].active;
            let slot = &mut g.st.players[o].slots[a as usize];
            slot.retreat_cost_increase_next_turn_pending = 1;
            slot.retreat_cost_increase_next_turn_attacker = Some(b.player);
            Ok(())
        }
        Effect::OpponentPokemonCannotAttackNextTurn { b, max_energy } => {
            let pl = &mut g.st.players[b.opponent as usize];
            match max_energy {
                None => pl.cannot_attack_turns_remaining = pl.cannot_attack_turns_remaining.max(1),
                Some(m) => {
                    pl.cannot_attack_max_energy = Some(m);
                    pl.cannot_attack_max_energy_turns_remaining = pl.cannot_attack_max_energy_turns_remaining.max(1);
                }
            }
            Ok(())
        }
        Effect::CoinFlipCancelTrainerPlay { b } => {
            let pl = &mut g.st.players[b.opponent as usize];
            pl.coin_flip_cancel_trainer_play_turns_remaining = pl.coin_flip_cancel_trainer_play_turns_remaining.max(1);
            Ok(())
        }
        Effect::PreventRetreat { b } => {
            // EffectOfAttackEffect.applyEffect(): the opponent's current Active.
            let o = b.opponent as usize;
            let a = g.st.players[o].active;
            g.st.players[o].slots[a as usize].cannot_retreat_next_turn = true;
            Ok(())
        }
        Effect::PreventDamage { b } => {
            // EffectOfAttackEffect.applyEffect(): the attacker's current Active.
            let p = b.player as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].prevent_damage_next_turn_pending = true;
            Ok(())
        }
        Effect::PreventDamageFiltered { b, filter } => {
            let p = b.player as usize;
            let a = g.st.players[p].active;
            let slot = &mut g.st.players[p].slots[a as usize];
            slot.prevent_damage_next_turn_pending = true;
            slot.prevent_damage_filter_pending = filter;
            Ok(())
        }
        Effect::PreventEffectsOfAttacks { b } => {
            let p = b.player as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].prevent_effects_of_attacks_next_turn_pending = true;
            Ok(())
        }
        Effect::ThisPokemonHasNoWeakness { b } => {
            let p = b.player as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].no_weakness_next_turn_pending = true;
            Ok(())
        }
        Effect::SelfPreventRetreat { b } => {
            let p = b.player as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_retreat_next_turn_pending = true;
            Ok(())
        }
        Effect::RetaliateOnDamage { b, damage, source_card } => {
            let p = b.player as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].retaliate_on_damage_next_turn_pending =
                Some(StoredRetaliate { damage, attack: b.attack, source_card, attacker: b.player });
            Ok(())
        }
        Effect::RetaliateDamage { b, damage } => {
            if damage > 0 {
                g.st.players[b.target.p as usize].slots[b.target.s as usize].damage += damage;
            }
            Ok(())
        }
        Effect::DiscardAttackerEnergyIfKnockedOut { b, source_card } => {
            let p = b.player as usize;
            let a = g.st.players[p].active;
            let slot = &mut g.st.players[p].slots[a as usize];
            slot.discard_attacker_energy_if_ko_next_turn = true;
            slot.discard_attacker_energy_if_ko_next_turn_pending = true;
            slot.discard_attacker_energy_if_ko_attack = Some(b.attack);
            slot.discard_attacker_energy_if_ko_source_card = Some(source_card);
            slot.discard_attacker_energy_if_ko_attacker = Some(b.player);
            Ok(())
        }
        Effect::ReduceDamage { b, reduction } => {
            // EffectOfAttackEffect.applyEffect(): the opponent's current Active.
            let o = b.opponent as usize;
            let a = g.st.players[o].active;
            g.st.players[o].slots[a as usize].attack_damage_reduction_next_turn = reduction.max(0);
            Ok(())
        }
        Effect::SwitchOutOpponentsActive { b, bench_target } => {
            if let Some(t) = bench_target {
                crate::engine::turn::switch_pokemon(g, b.opponent as usize, t.s)?;
            }
            Ok(())
        }
        Effect::OpponentPokemonCannotUseAttack { b, name } => {
            let o = b.opponent as usize;
            let a = g.st.players[o].active;
            g.st.players[o].slots[a as usize].blocked_attack_name_next_turn = Some(name);
            Ok(())
        }
        Effect::PreventAttackUntilLeavesActive { b, name } => {
            g.st.players[b.source.p as usize].slots[b.source.s as usize].blocked_attack_name_until_leaves_active = Some(name);
            Ok(())
        }
        Effect::DefendingPokemonTakesMoreDamage { b, damage_bonus } => {
            let o = b.opponent as usize;
            let a = g.st.players[o].active;
            let slot = &mut g.st.players[o].slots[a as usize];
            let already = slot.defending_extra_damage_next_turn > 0
                && !slot.defending_extra_damage_pending
                && slot.defending_extra_damage_attacker == Some(b.player);
            slot.defending_extra_damage_next_turn = damage_bonus;
            slot.defending_extra_damage_attacker = Some(b.player);
            if already {
                slot.defending_extra_damage_rearm_after_attack = true;
            } else {
                slot.defending_extra_damage_pending = true;
            }
            Ok(())
        }
        Effect::RemoveSpecialConditions { b, conditions } => {
            let slot = &mut g.st.players[b.target.p as usize].slots[b.target.s as usize];
            for &c in conditions.iter() {
                slot.special_conditions.retain(|x| *x != c);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Stats for a slot (used by cards).
pub fn check_stats(g: &mut Game, s: SlotRef) -> R<Effect> {
    let e = stats_effect(g, s);
    Ok(g.run_fx(e)?.0)
}
