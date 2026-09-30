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
    if g.st.turn == 1 && !ad.can_use_on_first_turn && !g.st.rules.attack_first_turn {
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
    // cannotAttackMaxEnergy / cannotUseAttacksNextTurn / blocked attack names /
    // cannotUseAttackUntilLeavesPlay / cannotUseGXAttacks /
    // coinFlipCancelAttackNextTurn: not modeled.

    let mut cost: Cost = SVec::new();
    for &c in ad.cost {
        cost.push(c);
    }
    let (ce, _) = g.run_fx(Effect::CheckAttackCost { p: p as u8, attack, cost })?;
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

fn finish_attack(g: &mut Game, f: AttackFrame) -> R {
    // Barrage: not modeled.
    g.release_fx(f.atk);
    g.release_fx(f.origin);
    g.run_fx(Effect::EndTurn { p: f.p })?;
    Ok(())
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
        // surviveOnTenHP: not modeled.
        let mut ab = b;
        ab.target = t;
        g.run_fx(Effect::AfterDamage { b: ab, damage })?;
    }
    Ok(())
}

pub fn reducer(g: &mut Game, id: EffId) -> R {
    // shouldPreventAttackEffects (preventEffectsOfAttacksNextTurn): not modeled.
    match *g.e(id) {
        Effect::PutDamage { b, damage, weakness_applied, .. } => {
            let t = b.target;
            if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
                crate::bail!("ILLEGAL_ACTION");
            }
            let opp = 1 - b.player as usize;
            let mut damage = damage;
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
            // preventDamageNextTurn: not modeled.
            let red = g.st.slot(t.p as usize, t.s).damage_reduction_next_turn;
            if red != 0 {
                damage = (damage - red).max(0);
            }
            if let Effect::PutDamage { damage: d, .. } = g.e_mut(id) {
                *d = damage;
            }
            apply_put_damage(g, id)
        }
        Effect::DealDamage { b, damage } => {
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
        Effect::AfterDamage { b, .. } => {
            g.st.players[b.target.p as usize].marker.add_to_state(DAMAGE_DEALT_MARKER);
            // Revenge traps: not modeled.
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
