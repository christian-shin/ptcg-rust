//! Copy-attack delegation: "choose 1 of the attacks of ... and use it as this attack" (Metronome, Foul Play, Seek Inspiration,
//! Gemstone Mimicry, Memory Helix).
//!
//! The Pokémon using the attack is the copycat and the copied attack is its own attack while it is used: the damage, the
//! effects and the Knock Outs after it are the copycat's (id2402, id453, id890, id2094), nothing of the source card applies
//! after the attack, and the copier needs none of the copied attack's Energy (ids 455, 2409). A copied attack that is
//! itself a copy attack (Seek Inspiration choosing Clefable's Metronome) is chosen and run like any other: it asks its own
//! choice (JP FAQ Slowking SCR 58, "Yes, you can"). The copied attack's lasting effects are lasting-table entries on the
//! copycat (`engine::lasting`).
//!
//! A copied attack runs as a fresh attack on the copycat carrying the source card's attack; after the normal fan-out of
//! every effect, `resolve_sessions` runs the source card's code with `me` = the copycat for the effects of the copied
//! attack's lifecycle. A session lasts while the copied attack is used, until the Knock Outs of the attack are done
//! (`attack::attack_over`).
//!
//! Rust model:
//! * A clone is an [`AttackRef`] on the *source* card with
//!   [`AttackRef::CLONE`] set and the session serial in bits 4..6, so it is
//!   distinct from the source's own attack and from the copycat's printed
//!   attacks, and `attack_def` still resolves it.
//! * Sessions live in [`Game::copy_sessions`] (so trial dispatch discards
//!   them).
//! * Delegation calls the source port's `reduce` with `me` = copycat while
//!   [`Game::deleg`] is set: `was_attack_used`/`my_attack` then compare with
//!   the clones, and card continuations the source code creates are tagged
//!   (`Cont::DelegCard`, `CoinCb::DelegCard`) so they resume in the source's
//!   port. Card handlers reached through nested effects run undelegated. The source's Abilities are
//!   blocked for the copycat while its code runs ([`Game::deleg`], `prefabs::is_ability_blocked`).
use crate::cards::{self, CardFrame};
use crate::effects::*;
use crate::engine::attack::{self, AttackFrame};
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, Debug)]
pub struct CopySession {
    pub copycat: CardId,
    pub source: CardId,
    pub serial: u8,
    /// Player index.
    pub player: u8,
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
        Effect::KnockOut { .. } => g.st.phase == GamePhase::Attack && g.st.active_player == s.player,
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

/// `resolveCopyAttackSessions` for the damage calculation's steps (they were the DealDamage / PutDamage effects; batch 8 C4,
/// the same order and the same sessions): after the game's own declarations, each live session's source card, run for
/// the copycat (the copied attack's lifecycle), adds its declarations of the step when the damaged Pokémon is the copycat
/// (step 5 also for the copied attack's own damage). A session whose copycat left play is dropped, as the effect's
/// fan-out dropped it.
pub fn delegate_damage(g: &mut Game, acc: &mut crate::spec::passive::DamageAcc, defender: bool) -> R {
    use crate::spec::passive::Modifier;
    let mut i = g.copy_sessions.len();
    while i > 0 {
        i -= 1;
        let s = match g.copy_sessions.get(i) {
            Some(s) => *s,
            // A session removed by a step's code: nothing to delegate.
            None => continue,
        };
        if !copycat_in_play(g, s.copycat) {
            g.copy_sessions.remove_at(i);
            continue;
        }
        let hit = target_includes(g, acc.b.target, s.copycat) || (defender && is_session_clone(&s, acc.b.attack));
        if !hit {
            continue;
        }
        let Some(spec) = cards::spec_for(g.st.cards[s.source as usize].def) else { continue };
        let fam = if defender { crate::decl::fam::DAMAGE_TAKEN } else { crate::decl::fam::DAMAGE_DEALT };
        if !spec.passives.iter().any(|ps| crate::decl::modifier_family(&ps.modifier) == Some(fam)) {
            continue;
        }
        let saved = g.deleg;
        g.deleg = Some(Deleg { copycat: s.copycat, source: s.source, serial: s.serial, attacks: true });
        let r = (|| -> R {
            for ps in spec.passives {
                match (&ps.modifier, defender) {
                    (Modifier::DamageDealt(d), false) => crate::spec::passive::damage_dealt_deal(g, s.copycat, ps.origin, d, acc)?,
                    (Modifier::DamageTaken(d), true) => crate::spec::passive::damage_taken_into(g, s.copycat, ps.origin, d, acc)?,
                    (Modifier::SurviveOnTen(x), true) => crate::spec::passive::survive_on_ten_into(g, s.copycat, ps.origin, x, acc)?,
                    _ => {}
                }
            }
            Ok(())
        })();
        g.deleg = saved;
        r?;
    }
    Ok(())
}

/// `resolveCopyAttackSessions` (runs after every effect's fan-out).
pub fn resolve_sessions(g: &mut Game, id: EffId) -> R {
    let mut i = g.copy_sessions.len();
    while i > 0 {
        i -= 1;
        let s = match g.copy_sessions.get(i) {
            Some(s) => *s,
            // Sessions removed by a nested effect: this one is gone, nothing to delegate.
            None => continue,
        };
        if !copycat_in_play(g, s.copycat) {
            g.copy_sessions.remove_at(i);
            continue;
        }
        if !should_delegate(g, &s, id) {
            continue;
        }
        delegate(g, s, id)?;
    }
    Ok(())
}

/// `openCopyAttackSession`; returns the clone of `source`'s attack `index`.
fn open_session(g: &mut Game, p: usize, copycat: CardId, source: CardId, index: u8) -> AttackRef {
    g.copy_sessions.retain(|s| s.copycat != copycat);
    g.copy_serial = g.copy_serial.wrapping_add(1);
    let serial = g.copy_serial;
    g.copy_sessions.push(CopySession { copycat, source, serial, player: p as u8 });
    // The attack the copycat used is the copying one (id2094: "the 'attack' it did was Genome Hacking"); the player's
    // record of it is the turn action's (`turn.rs`), never the clone.
    clone_ref(source, serial, index)
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
    /// The choice of the attack to copy is answered.
    ListChosen,
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
    /// The attack used another Pokémon's attack as its own (Memory Helix): the animation follows.
    pub then: Option<AttackFrame>,
    /// The Pokémon cards whose attacks the choice is among.
    pub cards: SVec<CardId, 64>,
    /// The choice can be cancelled.
    pub allow_cancel: bool,
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
            then: None,
            cards: SVec::new(),
            allow_cancel: false,
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

/// The choice of the attack to copy.
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

/// "Choose 1 of the attacks of these Pokémon and use it as this attack": the player chooses once. A copied attack that can't
/// be fully done is still used and ends (RULES.md "A copied attack is the copycat's only while it is used"); an error from its
/// program is an engine defect, as from any attack.
pub fn copy_attack_from_pokemon_list(g: &mut Game, atk: EffId, cards: &[CardId], allow_cancel: bool) -> R {
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
    f.allow_cancel = allow_cancel;
    for &c in cards {
        f.cards.push(c);
    }
    prompt_list(g, f)
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
        second_attack: false,
    });
    f.atk = atk;
    f.stage = CopyStage::AfterAttackFx;
    g.open_after_damage(atk);
    let r = g.reduce_effect(atk);
    finish_step(g, f, r, wait_if_prompts)
}

/// The stop of a copied attack that can't be done (a `Fail` of its program: "you can't use this attack during your first
/// turn", an unmet condition of its use): the copied attack is still used, does nothing more and ends.
pub const COPIED_ATTACK_ENDS: &str = "COPIED_ATTACK_ENDS";

/// Either suspend (`wait` returned true) or continue with the next stage; the end of a copied attack that can't be done
/// ([`COPIED_ATTACK_ENDS`]) is released and the copy ends; any other error from the copied attack's program is released and
/// propagates, like one from any attack.
fn finish_step(g: &mut Game, mut f: CopyFrame, r: R, wait: impl FnOnce(&mut Game, &mut CopyFrame) -> R<bool>) -> R {
    let r = r.and_then(|_| wait(g, &mut f));
    match r {
        Ok(true) => Ok(()),
        Ok(false) => next_stage(g, f),
        Err(e) if e.0 == COPIED_ATTACK_ENDS && f.then.is_none() => {
            g.release_fx(f.atk);
            Ok(())
        }
        Err(e) => {
            g.release_fx(f.atk);
            Err(e)
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
        CopyStage::ListChosen => Ok(()),
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
        // A wait item fired: continue after the stage that suspended.
        _ => next_stage(g, f),
    }
}

