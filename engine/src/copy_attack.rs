//! Copy-attack delegation: ports of `prefabs/copy-attack-delegation.ts` and
//! `prefabs/copy-attack-prefabs.ts`.
//!
//! Twinleaf runs a copied attack as a fresh `AttackEffect` carrying a clone
//! of the source card's attack (`cloneAttacks`). After the normal fan-out of
//! every effect, `resolveCopyAttackSessions` runs the source card's
//! `reduceEffect` with `this` = the copycat (and the copycat's `attacks`
//! temporarily replaced by the clones) for the effects of the copied attack's
//! lifecycle. The sessions are module state; they outlive the attack
//! (EndTurn / BetweenTurns / BeginTurn / damage to the copycat keep being
//! delegated) until 4 EndTurns passed or the copycat left play.
//!
//! Rust model:
//! * A clone is an [`AttackRef`] on the *source* card with
//!   [`AttackRef::CLONE`] set and the session serial in bits 4..6, so it is
//!   distinct from the source's own attack and from the copycat's printed
//!   attacks, and `attack_def` still resolves it.
//! * Sessions live in [`Game::copy_sessions`] (so trial dispatch discards
//!   them, unlike Twinleaf's module array).
//! * Delegation calls the source port's `reduce` with `me` = copycat while
//!   [`Game::deleg`] is set: `was_attack_used`/`my_attack` then compare with
//!   the clones, and card continuations the source code creates are tagged
//!   (`Cont::DelegCard`, `CoinCb::DelegCard`) so they resume in the source's
//!   port. Card handlers reached through nested effects run undelegated.
//!
//! Oracle caveat: trial dispatch (`legalTurnOptions`) snapshots the session
//! array, as Rust trials on game copies do. It also resolves info prompts,
//! including Twinleaf's "Coin flip animation" wait, whose coin was drawn from
//! the oracle's throwaway simulation stream; Rust stops at its coin chance
//! prompt. A delegated coin callback that throws on one outcome would
//! therefore make the oracle's legality depend on that stream (Annihilape's
//! Durable Body did for a power-less copycat until phase 4b: source code run
//! for the copycat now sees the copycat's Ability as blocked, see
//! [`Game::deleg`] and `prefabs::is_ability_blocked`).
use crate::cards::{self, CardFrame};
use crate::effects::*;
use crate::engine::attack::{self, AttackFrame};
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

/// `DEFAULT_END_TURN_BUDGET`.
pub const END_TURN_BUDGET: i8 = 4;

#[derive(Clone, Copy, Debug)]
pub struct CopySession {
    pub copycat: CardId,
    pub source: CardId,
    pub serial: u8,
    /// Player index (Twinleaf stores the id).
    pub player: u8,
    pub end_turns: i8,
    /// Runtime `barrage` flags of the cloned attacks (bit per index): the
    /// clones copy the source's flags at `cloneAttacks`; delegated
    /// `this.attacks[i].barrage = ...` writes land here, not on the copycat.
    pub barrage: u8,
}

/// Source code currently running as `.call(copycat)`.
#[derive(Clone, Copy, Debug)]
pub struct Deleg {
    pub copycat: CardId,
    pub source: CardId,
    pub serial: u8,
    /// Inside `delegateToSource` (the copycat's `attacks` are the clones);
    /// false inside callbacks created by it.
    pub attacks: bool,
}

pub fn clone_ref(source: CardId, serial: u8, index: u8) -> AttackRef {
    AttackRef { card: source, index: AttackRef::CLONE | ((serial & 7) << 4) | (index & 0x0F) }
}

fn is_session_clone(s: &CopySession, a: AttackRef) -> bool {
    a.is_clone() && a.card == s.source && (a.index >> 4) & 7 == s.serial & 7
}

fn copycat_in_play(g: &Game, c: CardId) -> bool {
    (0..2).any(|p| g.st.players[p].in_play().iter().any(|s| g.st.slot_pokemon(p, *s) == Some(c)))
}

fn target_includes(g: &Game, t: SlotRef, c: CardId) -> bool {
    g.st.slot(t.p as usize, t.s).cards.contains(c)
}

/// `shouldDelegateCopyAttackSession`.
fn should_delegate(g: &Game, s: &CopySession, id: EffId) -> bool {
    match *g.e(id) {
        Effect::EndTurn { .. } | Effect::BetweenTurns { .. } | Effect::BeginTurn { .. } => true,
        Effect::KnockOut { .. } => g.st.phase == GamePhase::Attack && g.st.active_player == s.player,
        Effect::DealDamage { b, .. } => target_includes(g, b.target, s.copycat),
        Effect::PutDamage { b, .. } => target_includes(g, b.target, s.copycat) || is_session_clone(s, b.attack),
        Effect::PlaceCounters { target, cause, .. } => target_includes(g, target, s.copycat) || cause.attack.map_or(false, |a| is_session_clone(s, a)),
        Effect::Damage { b, .. } => is_session_clone(s, b.attack),
        Effect::Attack { attack, .. } | Effect::BeforeDoingDamage { attack, .. } | Effect::AfterAttack { attack, .. } => is_session_clone(s, attack),
        _ => false,
    }
}

/// `delegateToSource`: the source's `reduceEffect.call(copycat, ...)`.
fn delegate(g: &mut Game, s: CopySession, id: EffId) -> R {
    let imp = match cards::impl_for(g.st.cards[s.source as usize].def) {
        Some(i) => i,
        None => return Ok(()),
    };
    if !imp.mask.has(g.e(id).kind()) {
        return Ok(());
    }
    let saved = g.deleg;
    g.deleg = Some(Deleg { copycat: s.copycat, source: s.source, serial: s.serial, attacks: true });
    let r = (imp.reduce)(g, s.copycat, id);
    g.deleg = saved;
    r
}

/// `resolveCopyAttackSessions` (runs after every effect's fan-out).
pub fn resolve_sessions(g: &mut Game, id: EffId) -> R {
    let mut i = g.copy_sessions.len();
    while i > 0 {
        i -= 1;
        let s = match g.copy_sessions.get(i) {
            Some(s) => *s,
            // `copyAttackSessions[i]` spliced away by a nested reduce.
            None => crate::bail!("TypeError: Cannot read properties of undefined (reading 'copycatCard')"),
        };
        if !copycat_in_play(g, s.copycat) {
            g.copy_sessions.remove_at(i);
            continue;
        }
        if !should_delegate(g, &s, id) {
            continue;
        }
        delegate(g, s, id)?;
        if let Effect::EndTurn { .. } = *g.e(id) {
            // `session.endTurnsRemaining -= 1` on the captured object, then
            // `copyAttackSessions.splice(i, 1)` by index (a no-op past the end).
            let mut left = s.end_turns - 1;
            if let Some(x) = g.copy_sessions.as_mut_slice().iter_mut().find(|x| x.copycat == s.copycat && x.serial == s.serial) {
                x.end_turns -= 1;
                left = x.end_turns;
            }
            if left <= 0 && i < g.copy_sessions.len() {
                g.copy_sessions.remove_at(i);
            }
        }
    }
    Ok(())
}

/// `openCopyAttackSession`; returns the clone of `source`'s attack `index`.
fn open_session(g: &mut Game, p: usize, copycat: CardId, source: CardId, index: u8) -> AttackRef {
    g.copy_sessions.retain(|s| s.copycat != copycat);
    g.copy_serial = g.copy_serial.wrapping_add(1);
    let serial = g.copy_serial;
    g.copy_sessions.push(CopySession { copycat, source, serial, player: p as u8, end_turns: END_TURN_BUDGET, barrage: g.st.cards[source as usize].attack_barrage });
    let clone = clone_ref(source, serial, index);
    g.st.player_last_attack[p] = Some((clone, copycat));
    g.st.player_last_attack_turn[p] = g.st.turn;
    clone
}

/// Resume a continuation created by delegated source code.
pub fn resume_deleg(g: &mut Game, card: CardId, source: CardId, serial: u8, frame: CardFrame, results: &[Res], coin: Option<bool>) -> R {
    let imp = match cards::impl_for(g.st.cards[source as usize].def) {
        Some(i) => i,
        None => return Ok(()),
    };
    let saved = g.deleg;
    g.deleg = Some(Deleg { copycat: card, source, serial, attacks: false });
    let r = match coin {
        Some(b) => match imp.coin {
            Some(c) => c(g, card, frame, b),
            None => Ok(()),
        },
        None => match imp.resume {
            Some(rs) => rs(g, card, frame, results),
            None => Ok(()),
        },
    };
    g.deleg = saved;
    r
}

// ---------------------------------------------------------------------------
// Generators

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CopyStage {
    /// COPY_ATTACK_FROM_POKEMON_LIST: ChooseAttackPrompt answered.
    ListChosen,
    /// COPY_ATTACK_VIA_ABILITY: ChooseAttackPrompt answered.
    AbilityChosen,
    AfterAttackFx,
    AfterBefore,
    AfterDeal,
    /// The Energy removals that waited for the damage have run (and their prompts resolved).
    AfterDamageEffects,
    AfterAfter,
    /// AfterAttackTriggersEffect resolved (and its prompts).
    AfterTriggers,
}

#[derive(Clone, Copy, Debug)]
pub struct CopyFrame {
    pub stage: CopyStage,
    pub p: u8,
    pub copycat: CardId,
    pub source: CardId,
    /// The clone being run.
    pub attack: AttackRef,
    /// The clone's `AttackEffect` (retained while the generator runs).
    pub atk: EffId,
    pub src_slot: SlotRef,
    /// COPY_ATTACK_FROM_POKEMON_LIST's try/catch: errors end the copy silently.
    pub catch: bool,
    /// useAttack continuation (delegateFrom path): the animation follows.
    pub then: Option<AttackFrame>,
    /// ChooseAttackPrompt cards.
    pub cards: SVec<CardId, 64>,
    /// `maxRetries` of COPY_ATTACK_FROM_POKEMON_LIST and the current attempt.
    pub max_retries: u8,
    pub retry: u8,
    /// `allowCancel` of the (re-issued) ChooseAttackPrompt.
    pub allow_cancel: bool,
    /// COPY_ATTACK_VIA_ABILITY: the prompt's blocked (card index, attack index) pairs.
    pub blocked: SVec<(u8, u8), 16>,
}

impl CopyFrame {
    fn new(stage: CopyStage, p: usize, copycat: CardId, src_slot: SlotRef) -> CopyFrame {
        CopyFrame {
            stage,
            p: p as u8,
            copycat,
            source: NO_CARD,
            attack: AttackRef { card: NO_CARD, index: 0 },
            atk: 0,
            src_slot,
            catch: false,
            then: None,
            cards: SVec::new(),
            max_retries: 1,
            retry: 0,
            allow_cancel: false,
            blocked: SVec::new(),
        }
    }
}

/// `noAttackLeftToCopy(pokemonCards, blocked)`: every attack of every Pokémon is
/// blocked (the prompt's validate rejects it), so a non-cancellable
/// ChooseAttackPrompt would have no valid answer. `blocked` holds the first
/// attack of each locked name, as the prompt's `find` resolves it.
fn no_attack_left_to_copy(g: &Game, cards: &[CardId], blocked: &[(u8, u8)]) -> bool {
    !cards.iter().enumerate().any(|(ci, c)| {
        let d = g.st.cdef(*c);
        d.is_pokemon() && (0..d.attacks.len()).any(|i| !blocked.iter().any(|b| b.0 as usize == ci && b.1 as usize == i))
    })
}

/// The ChooseAttackPrompt of one COPY_ATTACK_FROM_POKEMON_LIST attempt.
fn prompt_list(g: &mut Game, f: CopyFrame) -> R {
    let p = f.p as usize;
    let pc = f.cards;
    // A lock on the copying Pokémon ("can't use [Attack Name]") doesn't stop copying that attack: copying uses the
    // copying attack, not the copied one (ruling 381: Copycat copies Flare Strike turn after turn).
    let blocked: SVec<(u8, u8), 16> = SVec::new();
    if !f.allow_cancel && no_attack_left_to_copy(g, pc.as_slice(), blocked.as_slice()) {
        return Ok(());
    }
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_ATTACK_TO_COPY",
        PromptKind::ChooseAttack { cards: pc, allow_cancel: f.allow_cancel, blocked_message: "NOT_ENOUGH_ENERGY", blocked },
        Cont::CopyAttack(f),
    );
    Ok(())
}

/// `findPokemonCardForAttack`.
fn find_pokemon_for_attack(g: &Game, cards: &[CardId], a: AttackRef) -> Option<CardId> {
    let name = attack::attack_def(g, a).name;
    cards.iter().copied().find(|c| g.st.cdef(*c).is_pokemon() && (*c == a.card || g.st.cdef(*c).attacks.iter().any(|x| x.name == name)))
}

/// `findAttackIndex(source, attack)`.
fn find_attack_index(g: &Game, source: CardId, a: AttackRef) -> Option<u8> {
    if a.card == source && !a.is_clone() {
        return Some(a.idx() as u8);
    }
    let name = attack::attack_def(g, a).name;
    g.st.cdef(source).attacks.iter().position(|x| x.name == name).map(|i| i as u8)
}

/// `COPY_ATTACK_FROM_POKEMON_LIST` with `maxRetries`: a locked attack or an
/// error in the delegated attack re-issues the prompt until the attempts run out.
pub fn copy_attack_from_pokemon_list_retries(g: &mut Game, atk: EffId, cards: &[CardId], allow_cancel: bool, max_retries: u8) -> R {
    let (p, source) = match *g.e(atk) {
        Effect::Attack { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    if cards.is_empty() {
        return Ok(());
    }
    let copycat = match g.st.slot_pokemon(source.p as usize, source.s) {
        Some(c) => c,
        None => return Ok(()),
    };
    let mut f = CopyFrame::new(CopyStage::ListChosen, p, copycat, source);
    f.catch = true;
    f.max_retries = max_retries;
    f.allow_cancel = allow_cancel;
    for &c in cards {
        f.cards.push(c);
    }
    if max_retries == 0 {
        return Ok(());
    }
    prompt_list(g, f)
}

/// Push onto a blocked list of at most 16 entries (a full list can only miss a
/// blocked attack, which is then caught like any attack that throws).
fn push_blocked(v: &mut SVec<(u8, u8), 16>, e: (u8, u8)) {
    if v.len() < 16 {
        v.push(e);
    }
}

/// The ChooseAttackPrompt of COPY_ATTACK_VIA_ABILITY (`promptAttackToCopyViaAbility`).
fn prompt_ability(g: &mut Game, f: CopyFrame) -> R {
    let id = g.player_id(f.p as usize);
    g.prompt(
        id,
        "CHOOSE_ATTACK_TO_COPY",
        PromptKind::ChooseAttack { cards: f.cards, allow_cancel: true, blocked_message: "NOT_ENOUGH_ENERGY", blocked: f.blocked },
        Cont::CopyAttack(f),
    );
    Ok(())
}

/// useAttack's `delegateFrom` branch: `runDelegatedCopiedAttackGenerator`
/// (skipLog), then the attack animation.
pub fn run_delegated_from_use_attack(g: &mut Game, af: AttackFrame, copycat: CardId, source: CardId) -> R {
    let index = match find_attack_index(g, source, af.attack) {
        Some(i) => i,
        None => return attack::animation(g, af),
    };
    let mut f = CopyFrame::new(CopyStage::AfterAttackFx, af.p as usize, copycat, af.attacking);
    f.then = Some(af);
    start_delegated(g, f, source, index)
}

fn start_delegated(g: &mut Game, mut f: CopyFrame, source: CardId, index: u8) -> R {
    let p = f.p as usize;
    let clone = open_session(g, p, f.copycat, source, index);
    f.source = source;
    f.attack = clone;
    let damage = attack::attack_def(g, clone).damage;
    let atk = g.new_fx(Effect::Attack {
        p: p as u8,
        opp: (1 - p) as u8,
        attack: clone,
        damage,
        ignore_weakness: false,
        ignore_resistance: false,
        ignore_defender_effects: false,
        source: f.src_slot,
        barrage_used: false,
    });
    f.atk = atk;
    f.stage = CopyStage::AfterAttackFx;
    g.open_after_damage(atk);
    let r = g.reduce_effect(atk);
    finish_step(g, f, r, wait_if_prompts)
}

/// Apply the generator's error policy to a stage result, then either
/// suspend (`wait` returned true) or continue with the next stage.
fn finish_step(g: &mut Game, mut f: CopyFrame, r: R, wait: impl FnOnce(&mut Game, &mut CopyFrame) -> R<bool>) -> R {
    let r = r.and_then(|_| wait(g, &mut f));
    match r {
        Ok(true) => Ok(()),
        Ok(false) => next_stage(g, f),
        Err(e) => {
            g.release_fx(f.atk);
            if f.catch {
                if f.retry + 1 >= f.max_retries {
                    Ok(())
                } else {
                    f.retry += 1;
                    f.stage = CopyStage::ListChosen;
                    prompt_list(g, f)
                }
            } else {
                Err(e)
            }
        }
    }
}

fn next_stage(g: &mut Game, mut f: CopyFrame) -> R {
    let (p, opp) = (f.p, 1 - f.p);
    match f.stage {
        CopyStage::AfterAttackFx => {
            f.stage = CopyStage::AfterBefore;
            let r = g.run_fx(Effect::BeforeDoingDamage { attack_effect: f.atk, p, opp, attack: f.attack }).map(|_| ());
            finish_step(g, f, r, wait_if_prompts)
        }
        CopyStage::AfterBefore => {
            f.stage = CopyStage::AfterDeal;
            let damage = match *g.e(f.atk) {
                Effect::Attack { damage, .. } => damage,
                _ => 0,
            };
            if damage > 0 {
                let target = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
                let b = AtkBase { attack_effect: f.atk, player: p, opponent: opp, attack: f.attack, source: f.src_slot, target, cause: crate::cause::Cause::of_attack_at(g, p, f.attack, f.src_slot) };
                let r = crate::engine::damage::deal(g, b, damage, true);
                return finish_step(g, f, r, wait_if_prompts);
            }
            next_stage(g, f)
        }
        CopyStage::AfterDeal => {
            // `RUN_AFTER_DAMAGE_EFFECTS`: Energy removed as an effect of the attack leaves after the damage.
            f.stage = CopyStage::AfterDamageEffects;
            let r = g.run_after_damage(f.atk);
            finish_step(g, f, r, wait_if_prompts)
        }
        CopyStage::AfterDamageEffects => {
            f.stage = CopyStage::AfterAfter;
            // Tenacious Body / Durable Body: the coin is flipped after all the damage is done (ruling 1770).
            let r = crate::prefabs::resolve_survive_coin_flips(g).and_then(|_| g.run_fx(Effect::AfterAttack { p, opp, attack: f.attack, atk: f.atk }).map(|_| ()));
            finish_step(g, f, r, wait_if_prompts)
        }
        CopyStage::AfterAfter => {
            f.stage = CopyStage::AfterTriggers;
            let r = g.run_fx(Effect::AfterAttackTriggers { p, opp, attack: f.attack }).map(|_| ());
            finish_step(g, f, r, wait_if_prompts)
        }
        CopyStage::AfterTriggers => {
            // Step 7: one trigger at a time; a trigger that opens a prompt is answered before the next one.
            while g.attack_triggers_pending(f.atk) {
                let r = g.resolve_next_attack_trigger(f.atk);
                if r.is_err() || g.has_prompts() {
                    return finish_step(g, f, r, wait_if_prompts);
                }
            }
            g.close_attack_triggers(f.atk);
            g.release_fx(f.atk);
            match f.then {
                Some(af) => attack::animation(g, af),
                None => Ok(()),
            }
        }
        CopyStage::ListChosen | CopyStage::AbilityChosen => Ok(()),
    }
}

fn wait_if_prompts(g: &mut Game, f: &mut CopyFrame) -> R<bool> {
    if g.has_prompts() {
        g.wait_prompt(Cont::CopyAttack(*f));
        return Ok(true);
    }
    Ok(false)
}

pub fn resume(g: &mut Game, f: CopyFrame, res: Res) -> R {
    match f.stage {
        CopyStage::ListChosen => {
            let a = match res {
                Res::Attack(a) => a,
                _ => return Ok(()),
            };
            if attack::attack_def(g, a).copycat_attack {
                return Ok(());
            }
            let source = match find_pokemon_for_attack(g, f.cards.as_slice(), a) {
                Some(c) => c,
                None => return Ok(()),
            };
            let index = match find_attack_index(g, source, a) {
                Some(i) => i,
                None => return Ok(()),
            };
            start_delegated(g, f, source, index)
        }
        CopyStage::AbilityChosen => {
            let a = match res {
                Res::Attack(a) => a,
                _ => return Ok(()),
            };
            let source = match find_pokemon_for_attack(g, f.cards.as_slice(), a) {
                Some(c) => c,
                None => return Ok(()),
            };
            let p = f.p as usize;
            let active = SlotRef::new(p, g.st.players[p].active);
            let phase = g.st.phase;
            match g.run_fx(Effect::UseAttack { p: f.p, attack: a, source: active, ignore_status_conditions: false, barrage_used: false, delegate_from: Some(source) }) {
                Ok(_) => Ok(()),
                // `catch (error)`: a GameError (not a TypeError) means the chosen attack
                // cannot be used after all (its own conditions): choose another.
                Err(e) if !e.0.starts_with("TypeError") => {
                    g.st.phase = phase;
                    let index = f.cards.iter().position(|c| *c == a.card).unwrap_or(0) as u8;
                    let mut nf = f;
                    push_blocked(&mut nf.blocked, (index, a.idx() as u8));
                    prompt_ability(g, nf)
                }
                Err(e) => Err(e),
            }
        }
        // A wait item fired: continue after the stage that suspended.
        _ => next_stage(g, f),
    }
}

/// A runtime write `this.attacks[i].barrage = ...` by card code running as
/// `me`. While delegated source code runs with the clones installed as the
/// copycat's `attacks`, the write lands on the clone (the session's flags);
/// otherwise on `me`'s own attack (`attack_barrage`, shown in the canonical
/// state through `attack_barrage_shown`). `f(barrage, shown)` edits the
/// target's flag bits; the clone has no canonical `shown` bit.
pub fn write_barrage(g: &mut Game, me: CardId, f: impl FnOnce(&mut u8, &mut u8)) {
    if let Some(d) = g.deleg {
        if d.copycat == me && d.attacks {
            if let Some(s) = g.copy_sessions.as_mut_slice().iter_mut().find(|x| x.copycat == d.copycat && x.serial == d.serial) {
                let mut dummy = 0u8;
                f(&mut s.barrage, &mut dummy);
            }
            return;
        }
    }
    let inst = &mut g.st.cards[me as usize];
    f(&mut inst.attack_barrage, &mut inst.attack_barrage_shown);
}
