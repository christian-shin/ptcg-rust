//! Copied attacks: `copy-attack-delegation.ts` and `copy-attack-prefabs.ts`.
//!
//! Twinleaf runs a copied attack as a fresh `AttackEffect` for a clone of the
//! source card's attack, and keeps a "session" that re-runs the source card's
//! `reduceEffect` bound to the copycat (`sourceCard.reduceEffect.call(copycat)`,
//! with `copycat.attacks` temporarily set to the clones) after every effect of
//! the copied attack's lifecycle.
//!
//! Twinleaf keeps the sessions in a module-level array; here they live in
//! [`Game`] so clones of a game stay independent. A cloned attack is an
//! [`AttackRef`] on the source card whose index carries a clone generation in
//! its high bits, so it never equals a printed attack (object identity in TS).
//!
//! Oracle caveat: the oracle's trial-dispatch rollback (`legalTurnOptions`)
//! does not snapshot the module-level session array, so every trial of a turn
//! action that reaches `EndTurnEffect` (pass, attacks without prompts) spends
//! a session's end-turn budget and can drop sessions. Rust trials run on game
//! copies and do not. The difference is only observable when a delegated
//! source card reacts to effects outside the copied attack itself.

use crate::cards::{self, CardFrame};
use crate::effects::*;
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

/// `CopyAttackSession`.
#[derive(Clone, Copy, Debug)]
pub struct CopySession {
    /// Object identity of the session.
    pub id: u16,
    pub copycat: CardId,
    pub source: CardId,
    /// Clone generation of this session's `clonedAttacks`.
    pub gen: u8,
    pub player_id: u8,
    pub end_turns_remaining: i8,
}

/// The copycat's `attacks` are the session's clones while this is set
/// (`withTemporaryDelegatedAttacks`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Delegating {
    pub copycat: CardId,
    pub source: CardId,
    pub gen: u8,
}

const DEFAULT_END_TURN_BUDGET: i8 = 4;

/// The clone of `source`'s attack `ai` in generation `gen`.
pub fn cloned(source: CardId, gen: u8, ai: usize) -> AttackRef {
    AttackRef { card: source, index: (gen << 4) | ai as u8 }
}

fn is_cloned_attack(s: &CopySession, a: AttackRef) -> bool {
    a.card == s.source && a.index >> 4 == s.gen
}

fn is_copycat_in_play(g: &Game, copycat: CardId) -> bool {
    (0..2).any(|p| g.st.players[p].in_play().iter().any(|s| g.st.slot_pokemon(p, *s) == Some(copycat)))
}

fn target_includes_copycat(g: &Game, t: SlotRef, copycat: CardId) -> bool {
    g.st.slot(t.p as usize, t.s).cards.contains(copycat)
}

/// `shouldDelegateCopyAttackSession`.
fn should_delegate(g: &Game, s: &CopySession, e: &Effect) -> bool {
    match *e {
        Effect::EndTurn { .. } | Effect::BetweenTurns { .. } | Effect::BeginTurn { .. } => true,
        Effect::KnockOut { .. } => {
            g.st.phase == GamePhase::Attack && g.st.players[g.st.active_player as usize].id == s.player_id
        }
        Effect::DealDamage { b, .. } => target_includes_copycat(g, b.target, s.copycat),
        Effect::PutDamage { b, .. } | Effect::PutCounters { b, .. } => {
            target_includes_copycat(g, b.target, s.copycat) || is_cloned_attack(s, b.attack)
        }
        Effect::AfterDamage { b, .. } => is_cloned_attack(s, b.attack),
        Effect::Attack { attack, .. } | Effect::BeforeDoingDamage { attack, .. } | Effect::AfterAttack { attack, .. } => {
            is_cloned_attack(s, attack)
        }
        _ => false,
    }
}

/// `resolveCopyAttackSessions`: run after every effect's propagation.
pub fn resolve_sessions(g: &mut Game, id: EffId) -> R {
    if g.copy_sessions.is_empty() {
        return Ok(());
    }
    let mut i = g.copy_sessions.len();
    while i > 0 {
        i -= 1;
        let s = match g.copy_sessions.get(i) {
            Some(s) => *s,
            // `copyAttackSessions[i]` spliced away by a nested reduce.
            None => crate::bail!("TypeError: Cannot read properties of undefined (reading 'copycatCard')"),
        };
        if !is_copycat_in_play(g, s.copycat) {
            g.copy_sessions.remove_at(i);
            continue;
        }
        let e = *g.e(id);
        if !should_delegate(g, &s, &e) {
            continue;
        }
        delegate_to_source(g, &s, id)?;
        if matches!(e, Effect::EndTurn { .. }) {
            let mut left = s.end_turns_remaining - 1;
            if let Some(cur) = g.copy_sessions.as_mut_slice().iter_mut().find(|x| x.id == s.id) {
                cur.end_turns_remaining -= 1;
                left = cur.end_turns_remaining;
            }
            if left <= 0 && i < g.copy_sessions.len() {
                g.copy_sessions.remove_at(i);
            }
        }
    }
    Ok(())
}

/// `delegateToSource`: the source card's handler with `this` = copycat.
fn delegate_to_source(g: &mut Game, s: &CopySession, id: EffId) -> R {
    let imp = match cards::impl_for(g.st.cards[s.source as usize].def) {
        Some(i) => i,
        None => return Ok(()),
    };
    let kind = g.e(id).kind();
    if imp.mask & (1u128 << kind) == 0 {
        return Ok(());
    }
    let prev = g.delegating;
    g.delegating = Some(Delegating { copycat: s.copycat, source: s.source, gen: s.gen });
    let r = (imp.reduce)(g, s.copycat, id);
    g.delegating = prev;
    r
}

/// `openCopyAttackSession`: returns the clone generation.
fn open_session(g: &mut Game, p: usize, copycat: CardId, source: CardId, ai: usize) -> u8 {
    g.copy_sessions.retain(|s| s.copycat != copycat);
    g.copy_gen = g.copy_gen % 15 + 1;
    g.copy_serial = g.copy_serial.wrapping_add(1);
    let gen = g.copy_gen;
    g.copy_sessions.push(CopySession {
        id: g.copy_serial,
        copycat,
        source,
        gen,
        player_id: g.player_id(p),
        end_turns_remaining: DEFAULT_END_TURN_BUDGET,
    });
    g.st.player_last_attack[p] = Some((cloned(source, gen, ai), copycat));
    gen
}

// ---------------------------------------------------------------------------
// COPY_ATTACK_FROM_POKEMON_LIST

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CopyStage {
    /// ChooseAttackPrompt answered.
    Chosen,
    AfterAttackEffect,
    AfterBeforeDoingDamage,
    AfterDealDamage,
    AfterAfterAttack,
}

#[derive(Clone, Copy, Debug)]
pub struct CopyFrame {
    pub stage: CopyStage,
    pub p: u8,
    pub copycat: CardId,
    /// `effect.source` of the copycat's AttackEffect.
    pub source_slot: SlotRef,
    pub cards: SVec<CardId, 8>,
    pub disallow_copycat: bool,
    /// The copied AttackEffect (after `Chosen`).
    pub atk: EffId,
    pub attack: AttackRef,
}

pub struct CopyOpts {
    pub allow_cancel: bool,
    pub disallow_copycat_attack: bool,
}

/// `COPY_ATTACK_FROM_POKEMON_LIST(store, state, effect, pokemonCards, options)`
/// with `maxRetries` 1, no `blocked`, default prompt player.
pub fn copy_attack_from_pokemon_list(g: &mut Game, atk: EffId, pokemon: &[CardId], o: CopyOpts) -> R {
    let (p, source) = match *g.e(atk) {
        Effect::Attack { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    if pokemon.is_empty() {
        return Ok(());
    }
    let copycat = match g.st.slot_pokemon(source.p as usize, source.s) {
        Some(c) => c,
        None => return Ok(()),
    };
    // blockCannotUseAttacksNextTurn: `cannotUseAttacksNextTurn` is not modeled
    // (no ported card sets it), so nothing is blocked.
    let mut cards: SVec<CardId, 8> = SVec::new();
    for &c in pokemon {
        cards.push(c);
    }
    let f = CopyFrame {
        stage: CopyStage::Chosen,
        p: p as u8,
        copycat,
        source_slot: source,
        cards,
        disallow_copycat: o.disallow_copycat_attack,
        atk: 0,
        attack: AttackRef { card: NO_CARD, index: 0 },
    };
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_ATTACK_TO_COPY",
        PromptKind::ChooseAttack { cards, allow_cancel: o.allow_cancel, blocked_message: "NOT_ENOUGH_ENERGY", blocked: SVec::new() },
        Cont::CopyAttack(f),
    );
    Ok(())
}

/// `findPokemonCardForAttack`: first card whose attacks include one of that name.
fn find_pokemon_card_for_attack(g: &Game, cards: &[CardId], name: &str) -> Option<CardId> {
    cards.iter().copied().find(|c| {
        let d = g.st.cdef(*c);
        d.is_pokemon() && d.attacks.iter().any(|a| a.name == name)
    })
}

pub fn resume(g: &mut Game, mut f: CopyFrame, results: &[Res]) -> R {
    if f.stage == CopyStage::Chosen {
        let chosen = match results.first().copied().unwrap_or(Res::Null) {
            Res::Attack(a) => a,
            _ => return Ok(()),
        };
        // isAttackLockedNextTurn: not modeled (see above).
        let ad = &g.st.cdef(chosen.card).attacks[chosen.ai()];
        if f.disallow_copycat && ad.copycat_attack {
            return Ok(());
        }
        let name = ad.name;
        let source = match find_pokemon_card_for_attack(g, f.cards.as_slice(), name) {
            Some(c) => c,
            None => return Ok(()),
        };
        // findAttackIndex: by reference, else by name.
        let ai = if source == chosen.card {
            chosen.ai()
        } else {
            match g.st.cdef(source).attacks.iter().position(|a| a.name == name) {
                Some(i) => i,
                None => return Ok(()),
            }
        };
        let p = f.p as usize;
        let gen = open_session(g, p, f.copycat, source, ai);
        let attack = cloned(source, gen, ai);
        let damage = g.st.cdef(source).attacks[ai].damage;
        let atk = g.new_fx(Effect::Attack {
            p: f.p,
            opp: (1 - p) as u8,
            attack,
            damage,
            ignore_weakness: false,
            ignore_resistance: false,
            source: f.source_slot,
            barrage_used: false,
        });
        f.atk = atk;
        f.attack = attack;
        // `try { yield* runDelegatedCopiedAttackGenerator } catch { return }`.
        if g.reduce_effect(atk).is_err() {
            g.release_fx(atk);
            return Ok(());
        }
        return step(g, f, CopyStage::AfterAttackEffect);
    }
    let r = match f.stage {
        CopyStage::AfterAttackEffect => before_doing_damage(g, f),
        CopyStage::AfterBeforeDoingDamage => deal_damage(g, f),
        CopyStage::AfterDealDamage => after_attack(g, f),
        _ => {
            g.release_fx(f.atk);
            return Ok(());
        }
    };
    if r.is_err() {
        g.release_fx(f.atk);
    }
    Ok(())
}

/// `if (store.hasPrompts()) yield store.waitPrompt(...)`, else continue.
fn step(g: &mut Game, mut f: CopyFrame, next: CopyStage) -> R {
    if g.has_prompts() {
        f.stage = next;
        g.wait_prompt(Cont::CopyAttack(f));
        return Ok(());
    }
    f.stage = next;
    resume(g, f, &[])
}

fn before_doing_damage(g: &mut Game, f: CopyFrame) -> R {
    let p = f.p;
    g.run_fx(Effect::BeforeDoingDamage { attack_effect: f.atk, p, opp: 1 - p, attack: f.attack })?;
    step(g, f, CopyStage::AfterBeforeDoingDamage)
}

fn deal_damage(g: &mut Game, f: CopyFrame) -> R {
    let (damage, source) = match *g.e(f.atk) {
        Effect::Attack { damage, source, .. } => (damage, source),
        _ => (0, f.source_slot),
    };
    if damage > 0 {
        let o = 1 - f.p as usize;
        let target = SlotRef::new(o, g.st.players[o].active);
        let b = AtkBase { attack_effect: f.atk, player: f.p, opponent: o as u8, attack: f.attack, source, target };
        g.run_fx(Effect::DealDamage { b, damage })?;
        return step(g, f, CopyStage::AfterDealDamage);
    }
    after_attack(g, f)
}

fn after_attack(g: &mut Game, f: CopyFrame) -> R {
    let p = f.p;
    g.run_fx(Effect::AfterAttack { p, opp: 1 - p, attack: f.attack })?;
    step(g, f, CopyStage::AfterAfterAttack)
}

/// Frames created while a source handler runs for the copycat resume that
/// source's port (the TS closure captured the source method).
pub fn route_cont(g: &Game, c: Cont) -> Cont {
    match (g.delegating, c) {
        (Some(d), Cont::Card { card, frame }) if card == d.copycat => Cont::CardAs { imp: d.source, card, frame },
        _ => c,
    }
}

pub fn resume_card_as(g: &mut Game, imp: CardId, card: CardId, f: CardFrame, results: &[Res]) -> R {
    match cards::impl_for(g.st.cards[imp as usize].def).and_then(|i| i.resume) {
        Some(r) => r(g, card, f, results),
        None => Ok(()),
    }
}
