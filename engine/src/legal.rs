//! Legality from declared checks, not from playing each candidate on a fork (S4, "legality without trials").
//!
//! [`Ctx`] is the per-decision derived context: everything a candidate's answer needs that is not a plain state
//! read. Checked reads (the effects through the cards: the attack list and costs, provided Energy, the
//! retreat cost, evolution timing, Ability lock probes, a card's `needs`) are made on ONE scratch copy of the
//! game per decision, lazily, and cached (the Energy a Pokémon provides is read once, whatever number of
//! attacks and retreats ask for it). Plain state reads go straight to the game.
//!
//! [`legal_fast`] answers one action: `Some(legal)` when every rule that could reject it is declared and was
//! evaluated, `None` when one is not (the caller then plays the action on a copy, a counted fallback). It
//! calls the same `can_x()` checks the executing reducers call (`turn.rs`, `play.rs`, `retreat.rs`,
//! `attack.rs`), the same play-lock query (`passive::play_locked`) and the same declared card checks
//! (`spec::run`: `needs` / implied preconditions, once per turn, leading `Fail` steps), so the answer and the
//! execution can't drift apart. `PTCG_VERIFY_LEGAL=1` compares every answer with the full trial.
//!
//! What each action kind checks:
//! * Energy attach: the turn rule (`can_attach_energy`), the target, the player's Energy-play flags, the play
//!   locks (`LockedAction::AttachEnergy`), the Energy card's own attach guard.
//! * Pokémon (Basic, evolve): the event the play produces (`enter::hand_play_view`), its locks
//!   (`event_locked`), and for an Evolve the evolution rules (`enter::evolve_rules`: evolves from, the
//!   rule's limits with the permissions, the restrictions).
//! * Item / Supporter / Stadium / Tool: the turn rules and flags (`can_play_*`), the play locks, and the
//!   card's declared `needs` and implied preconditions evaluated in the state its effect would see.
//! * Attack: the attack list read, `can_attack_pre/post`, the max-Energy rule, blocks (`BlockAttack`), the
//!   leading `Fail` steps, the cost against the provided Energy.
//! * Ability: the power list read, the core use rule, the lock probe, once per turn, `needs` and implied
//!   preconditions.
//! * Use Stadium: `can_use_stadium`, the play locks, the Stadium's own `needs`.
//! * Retreat: `can_retreat`, the play locks, the cost against the provided Energy.
//! * Pass: always.

use crate::carddb::DefId;
use crate::effects::*;
use crate::engine::{attack, play, retreat, turn};
use crate::game::{Action, Fork, Game};
use crate::list::*;
use crate::prompts::get_target;
use crate::state::AttackRef;
use crate::spec::passive::{self, LockedAction};
use crate::spec::run::Gate;
use crate::types::*;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Which cards declare something legality reads

const F_BLOCK_USE: u8 = 1;
const F_BLOCK_ATTACK: u8 = 2;
/// A lock over events (`LockDecl::forbids`).
const F_EVENT_LOCK: u8 = 4;
const F_ATTACK_FAIL: u8 = 8;
const F_ATTACH_GUARD: u8 = 16;
const F_PLAY_SPEC: u8 = 32;
const F_PLAYS_AS_POKEMON: u8 = 64;

fn def_flags(def: DefId) -> u8 {
    static T: OnceLock<Vec<u8>> = OnceLock::new();
    T.get_or_init(|| {
        (0..crate::carddb::cards().len())
            .map(|d| {
                let Some(spec) = crate::cards::spec_for(d as DefId) else { return 0 };
                let mut f = 0;
                for ps in spec.passives {
                    f |= match &ps.modifier {
                        passive::Modifier::BlockUse(b) if !b.lock.forbids.is_never() => F_BLOCK_USE | F_EVENT_LOCK,
                        passive::Modifier::BlockUse(_) => F_BLOCK_USE,
                        passive::Modifier::BlockAttack(_) => F_BLOCK_ATTACK,
                        passive::Modifier::AttachGuard(_) => F_ATTACH_GUARD,
                        _ => 0,
                    };
                }
                if crate::spec::run::spec_has_fail(spec) {
                    f |= F_ATTACK_FAIL;
                }
                if spec.play.is_some() {
                    f |= F_PLAY_SPEC;
                }
                if crate::spec::run::plays_as_pokemon(spec) {
                    f |= F_PLAYS_AS_POKEMON;
                }
                f
            })
            .collect()
    })[def as usize]
}

/// The cards of the game that declare a lock-like check, by kind (found once per decision).
#[derive(Default)]
struct Sources {
    block_use: SVec<CardId, 120>,
    block_attack: SVec<CardId, 120>,
    event_lock: SVec<CardId, 120>,
}

// ---------------------------------------------------------------------------
// The per-decision context

pub struct Ctx<'a> {
    pub g: &'a Game,
    p: usize,
    scratch: Option<Fork>,
    sources: Option<Sources>,
    /// The checked attack list and the available attacks built from it.
    attacks: Option<Result<(turn::CheckedAttacks, SVec<(AttackRef, bool), 64>), ()>>,
    /// CheckProvidedEnergy per slot.
    provided: SVec<(crate::state::SlotId, Result<EnergyMap, ()>), { crate::state::MAX_SLOTS }>,
    /// CheckRetreatCost.
    retreat_cost: Option<Result<Cost, ()>>,
    /// The reason of the last `None` from `legal_fast` (the fallback table).
    pub why: &'static str,
}

impl<'a> Ctx<'a> {
    pub fn new(g: &'a Game) -> Ctx<'a> {
        Ctx { g, p: g.st.active_player as usize, scratch: None, sources: None, attacks: None, provided: SVec::new(), retreat_cost: None, why: "" }
    }

    /// The scratch game checked reads run on (made on first use, one per decision).
    fn sc(&mut self) -> &mut Game {
        let g = self.g;
        &mut **self.scratch.get_or_insert_with(|| g.fork())
    }

    fn none(&mut self, why: &'static str) -> Option<bool> {
        self.why = why;
        None
    }

    fn sources(&mut self) -> &Sources {
        if self.sources.is_none() {
            let g = self.g;
            let mut s = Sources::default();
            for c in 0..g.st.n_cards {
                let f = def_flags(g.st.cards[c as usize].def);
                if f & F_BLOCK_USE != 0 {
                    s.block_use.push(c);
                }
                if f & F_BLOCK_ATTACK != 0 {
                    s.block_attack.push(c);
                }
                if f & F_EVENT_LOCK != 0 {
                    s.event_lock.push(c);
                }
            }
            self.sources = Some(s);
        }
        self.sources.as_ref().unwrap()
    }

    // --- the derived reads ---------------------------------------------------

    /// The attacks the active player may use: the `CheckPokemonAttacks` read (made only when a card or a
    /// Tool can change the list), and the list `AttackAction` builds from it.
    pub fn attack_list(&mut self) -> Result<&(turn::CheckedAttacks, SVec<(AttackRef, bool), 64>), ()> {
        if self.attacks.is_none() {
            let g = self.g;
            let p = self.p;
            let checked = if g.kinds_present.has(k::CHECK_POKEMON_ATTACKS) {
                crate::derived::attacks(self.sc(), p).map_err(|_| ())
            } else {
                // No handler: the read gives the seed (the Active Pokémon's Tool attacks).
                match turn::check_attacks_effect(g, p) {
                    Effect::CheckPokemonAttacks { attacks, copied, .. } => Ok((attacks, copied)),
                    _ => Ok((SVec::new(), SVec::new())),
                }
            };
            self.attacks = Some(checked.map(|c| {
                let all = turn::assemble_available_attacks(g, p, &c);
                (c, all)
            }));
        }
        self.attacks.as_ref().unwrap().as_ref().map_err(|_| ())
    }

    /// The Energy the Pokémon in `slot` provides (`CheckProvidedEnergy`), read once.
    fn provided(&mut self, slot: crate::state::SlotId) -> Result<EnergyMap, ()> {
        if let Some((_, r)) = self.provided.iter().find(|(s, _)| *s == slot) {
            return r.clone();
        }
        let p = self.p;
        let r = crate::derived::provided_energy(self.sc(), p, SlotRef::new(p, slot)).map_err(|_| ());
        self.provided.push((slot, r.clone()));
        r
    }

    /// The effective Retreat Cost (`CheckRetreatCost`), read once.
    fn retreat_cost(&mut self) -> Result<Cost, ()> {
        if let Some(r) = &self.retreat_cost {
            return r.clone();
        }
        let p = self.p;
        let r = crate::derived::retreat_cost(self.sc(), p).map_err(|_| ());
        self.retreat_cost = Some(r.clone());
        r
    }

    /// Is `action` with `card` locked by a declared play lock (`passive::play_locked`)?
    fn play_locked(&mut self, card: CardId, action: &[LockedAction]) -> bool {
        let p = self.p;
        if !action.iter().any(|a| self.g.kinds_present.has(a.kind())) || self.sources().block_use.is_empty() {
            return false;
        }
        for i in 0..self.sources().block_use.len() {
            let src = self.sources().block_use[i];
            if passive::play_locked_by(self.sc(), src, p, card, action).is_some() {
                return true;
            }
        }
        false
    }

    /// Is the event forbidden by a declared lock (`passive::event_locked`, the same declarations: the locks over
    /// events, in play and lasting)?
    fn event_locked(&mut self, v: &crate::spec::event::EventView) -> bool {
        let g = self.g;
        if v.card.is_none() {
            return false;
        }
        let p = v.owner as usize;
        if g.kinds_present.has(crate::effects::k::DECLARES_EVENT_LOCK) {
            for i in 0..self.sources().event_lock.len() {
                let src = self.sources().event_lock[i];
                if !matches!(passive::event_locked_by(self.sc(), src, v), Ok(None)) {
                    return true;
                }
            }
        }
        if g.st.players[p].lasting_locks.iter().flatten().any(|l| !l.decl.forbids.is_never()) {
            return !matches!(passive::lasting_event_locked(self.sc(), v), Ok(None));
        }
        false
    }
}

/// Answer one action from the declared checks; `None` (with the reason in `ctx.why`) when it can't.
pub fn legal_fast(ctx: &mut Ctx, a: Action) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    match a {
        Action::Pass => Some(true),
        Action::PlayCard { hand_index, target } => {
            let Some(card) = g.st.players[p].hand.get(hand_index as usize) else { return ctx.none("unknown hand card") };
            let d = g.st.cdef(card);
            if d.is_energy() {
                fast_energy(ctx, card, target)
            } else if d.is_pokemon() {
                fast_pokemon(ctx, card, target)
            } else if d.is_trainer() {
                fast_trainer(ctx, card, target)
            } else {
                ctx.none("unknown card kind")
            }
        }
        Action::Attack { name, from } => fast_attack(ctx, name, from),
        Action::UseAbility { name, target } => fast_ability(ctx, name, target),
        Action::UseTrainerAbility { .. } => ctx.none("trainer ability"),
        Action::UseStadium => fast_use_stadium(ctx),
        Action::Retreat { bench_index } => fast_retreat(ctx, bench_index),
    }
}

// ---------------------------------------------------------------------------
// Energy

fn fast_energy(ctx: &mut Ctx, card: CardId, target: CardTarget) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    let Ok((t, _)) = turn::can_attach_energy(g, p, target) else { return Some(false) };
    // play_energy_reducer
    if g.st.slot_pokemon(t.p as usize, t.s).is_none() {
        return Some(false);
    }
    if passive::lasting_locked(g, p, Some(card), &[LockedAction::AttachEnergy]).is_some() {
        return Some(false);
    }
    if ctx.play_locked(card, &[LockedAction::AttachEnergy]) {
        return Some(false);
    }
    if def_flags(g.st.cards[card as usize].def) & F_ATTACH_GUARD != 0 && !matches!(passive::attach_guard_refuses(ctx.sc(), card, t), Ok(false)) {
        return Some(false);
    }
    Some(true)
}

// ---------------------------------------------------------------------------
// Pokémon

fn fast_pokemon(ctx: &mut Ctx, card: CardId, target: CardTarget) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    let Ok(t) = get_target(&g.st, p, target) else { return Some(false) };
    // The event the play produces (`enter::hand_play_view`) and the same checks its routine makes
    // (`enter::check_enter` / `check_evolve`): the locks, then the evolution rules.
    let Ok(v) = crate::engine::enter::hand_play_view(g, p, card, t) else { return Some(false) };
    if ctx.event_locked(&v) {
        return Some(false);
    }
    if v.kind == crate::spec::event::EventKind::EnterPlay {
        if crate::engine::enter::may_be_restricted(g, &v) {
            return Some(matches!(crate::engine::enter::restricted(ctx.sc(), &v), Ok(None)));
        }
        return Some(true);
    }
    let reach = crate::engine::enter::Reach::Next;
    match crate::engine::enter::evolve_rules_fast(g, &v, reach) {
        Some(ok) => Some(ok),
        None => Some(crate::engine::enter::evolve_rules(ctx.sc(), &v, reach).is_ok()),
    }
}

// ---------------------------------------------------------------------------
// Trainers

fn fast_trainer(ctx: &mut Ctx, card: CardId, target: CardTarget) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    let d = g.st.cdef(card);
    match d.trainer_type() {
        TrainerType::Item => {
            if play::can_play_item_with(g, p, Some(card)).is_err() || ctx.play_locked(card, &[LockedAction::PlayItem]) {
                return Some(false);
            }
            if def_flags(d_def(g, card)) & F_PLAY_SPEC == 0 {
                return Some(true);
            }
            // An Item played as a Pokémon (a Fossil) plays the card onto the first empty Bench slot.
            if def_flags(d_def(g, card)) & F_PLAYS_AS_POKEMON != 0 {
                match crate::prefabs::empty_bench_slots(g, p).as_slice().first() {
                    None => return Some(false),
                    Some(&s) => {
                        // EnterPlay by the rule from the hand, caused by the Trainer card (`Op::PlayAsPokemon`).
                        let t = SlotRef::new(p, s);
                        let cause = crate::cause::Cause::of_trainer(g, card, p as u8);
                        let v = crate::engine::enter::enter_view(g, card, t, crate::spec::event::RulesZone::Hand, crate::spec::event::EnterMode::Rule, cause);
                        if ctx.event_locked(&v) || (crate::engine::enter::may_be_restricted(g, &v) && !matches!(crate::engine::enter::restricted(ctx.sc(), &v), Ok(None))) {
                            return Some(false);
                        }
                    }
                }
            }
            // The Item has left the hand for the play area when its effect runs.
            let sc = ctx.sc();
            let (hand, supporter) = (sc.st.players[p].hand, sc.st.players[p].supporter);
            play::enter_item_play(sc, p, card);
            let e = sc.new_fx(Effect::Trainer { p: p as u8, card, target: None, via_attack: false });
            let ok = crate::spec::run::trainer_play_check(sc, card, p, e).is_ok();
            sc.release_fx(e);
            // The lists go back by assignment: the layout tracking (the dispatch index) learns it here.
            crate::list::mark_all(sc.st.players[p].hand.as_slice());
            crate::list::mark_all(sc.st.players[p].supporter.as_slice());
            sc.st.players[p].hand = hand;
            sc.st.players[p].supporter = supporter;
            crate::list::mark_all(hand.as_slice());
            crate::list::mark_all(supporter.as_slice());
            Some(ok)
        }
        TrainerType::Supporter => {
            if turn::can_play_supporter_card(g, p, card).is_err() || play::can_play_supporter_with(g, p, Some(card)).is_err() || ctx.play_locked(card, &[LockedAction::PlaySupporter]) {
                return Some(false);
            }
            if def_flags(d_def(g, card)) & F_PLAY_SPEC == 0 {
                return Some(true);
            }
            let sc = ctx.sc();
            let e = sc.new_fx(Effect::Trainer { p: p as u8, card, target: None, via_attack: false });
            let ok = crate::spec::run::trainer_play_check(sc, card, p, e).is_ok();
            sc.release_fx(e);
            Some(ok)
        }
        TrainerType::Stadium => Some(
            turn::can_play_stadium_card(g, p, card).is_ok() && play::can_play_stadium_with(g, p, Some(card)).is_ok() && !ctx.play_locked(card, &[LockedAction::PlayStadium]),
        ),
        TrainerType::Tool => {
            // No Tool declares a play spec: its effects are passives (a Tool with one would need the state
            // after it is attached).
            if def_flags(d_def(g, card)) & F_PLAY_SPEC != 0 {
                return ctx.none("tool with a play spec");
            }
            let t = get_target(&g.st, p, target).ok();
            let Ok(t) = turn::can_play_tool_card(t) else { return Some(false) };
            if ctx.play_locked(card, &[LockedAction::AttachTool]) {
                return Some(false);
            }
            Some(play::can_attach_tool(g, p, card, t).is_ok())
        }
    }
}

fn d_def(g: &Game, card: CardId) -> DefId {
    g.st.cards[card as usize].def
}

// ---------------------------------------------------------------------------
// Attacks

fn fast_attack(ctx: &mut Ctx, name: &'static str, from: Option<&'static str>) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    let found = match ctx.attack_list() {
        Err(()) => return Some(false),
        Ok((_, all)) => turn::find_attack(g, all.as_slice(), name, from),
    };
    let Some((attack, copied)) = found else { return Some(false) };
    if attack.is_clone() {
        return ctx.none("attack: a copy-attack clone");
    }
    // The first-turn flag an Ability writes as the attack is used (Meloetta ex).
    let granted = g.st.turn == 1 && g.kinds_present.has(k::USE_ATTACK) && passive::grants_first_turn_attack(ctx.sc(), p);
    let Ok(attacking) = attack::can_attack_pre(g, p, attack, false, granted) else { return Some(false) };
    let max_energy = if attack::attack_max_energy_applies(g, p) {
        match ctx.provided(attacking.s) {
            Ok(map) => Some(attack::max_energy_count(&map)),
            Err(()) => return Some(false),
        }
    } else {
        None
    };
    if attack::can_attack_post(g, p, attack, attacking, max_energy).is_err() {
        return Some(false);
    }
    // Blocks and preconditions the cards declare.
    if g.kinds_present.has(k::USE_ATTACK) || g.kinds_present.has(k::ATTACK) {
        let n = ctx.sources().block_attack.len();
        for i in 0..n {
            let src = ctx.sources().block_attack[i];
            if passive::attack_blocked_by(ctx.sc(), src, p) {
                return Some(false);
            }
        }
    }
    if def_flags(d_def(g, attack.card)) & F_ATTACK_FAIL != 0 {
        let ad = attack::attack_def(g, attack);
        let sc = ctx.sc();
        let e = sc.new_fx(Effect::Attack {
            p: p as u8,
            opp: (1 - p) as u8,
            attack,
            damage: ad.damage,
            ignore_weakness: false,
            ignore_resistance: false,
            ignore_defender_effects: false,
            source: attacking,
            barrage_used: false,
        });
        // (An attack copied from a Benched Pokémon runs its text as the copycat's.)
        let me = if copied { g.st.slot_pokemon(p, attacking.s).unwrap_or(attack.card) } else { attack.card };
        let gate = crate::spec::run::attack_gate(sc, me, attack, p, e);
        sc.release_fx(e);
        match gate {
            Ok(Gate::Open) => {}
            Ok(Gate::Closed) | Err(_) => return Some(false),
            Ok(Gate::Undeclared) => return ctx.none("attack: a Fail step after other steps"),
        }
    }
    // The cost against the Energy provided.
    let Ok(cost) = attack::attack_cost_read(ctx.sc(), p, attack) else { return Some(false) };
    let Ok(map) = ctx.provided(attacking.s) else { return Some(false) };
    Some(crate::energy::check_enough_energy(map.as_slice(), cost.as_slice()))
}

// ---------------------------------------------------------------------------
// Abilities

fn fast_ability(ctx: &mut Ctx, name: &'static str, target: CardTarget) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    let card = match turn::ability_source(g, p, target) {
        Err(_) => return Some(false),
        Ok(None) => return ctx.none("ability: no Pokémon"),
        Ok(Some(c)) => c,
    };
    // The printed power by name: the checked list only removes powers, so one that isn't printed, or that the
    // core rule refuses, is illegal whatever the lock reads say.
    let mut same = g.st.cdef(card).powers.iter().enumerate().filter(|(_, pw)| pw.name == name).map(|(i, _)| i);
    let Some(i) = same.next() else { return Some(false) };
    if same.next().is_some() {
        return ctx.none("ability: two powers with one name");
    }
    let mut power = PowerRef { card, index: i as u8 };
    if turn::can_use_ability_core(g, power, target.slot).is_err() || crate::spec::run::power_once_check(g, card, power.index, p).is_err() {
        return Some(false);
    }
    // The powers the card has now (CheckPokemonPowers).
    if g.kinds_present.has(k::CHECK_POKEMON_POWERS) {
        let mut powers: SVec<PowerRef, 8> = SVec::new();
        for i in 0..g.st.cdef(card).powers.len() {
            powers.push(PowerRef { card, index: i as u8 });
        }
        match ctx.sc().run_fx(Effect::CheckPokemonPowers { p: p as u8, target: card, powers }) {
            Ok((Effect::CheckPokemonPowers { powers: after, .. }, _)) => match after.iter().find(|r| g.st.cdef(r.card).powers[r.index as usize].name == name) {
                Some(r) => power = *r,
                None => return Some(false),
            },
            Ok(_) => {}
            Err(_) => return Some(false),
        }
    }
    let sc = ctx.sc();
    if attack::power_use_blocked(sc, p, power, card) {
        return Some(false);
    }
    let e = sc.new_fx(Effect::Power { p: p as u8, power, card, target: None, probe: false });
    let ok = crate::spec::run::power_check(sc, card, power.index, p, e).is_ok();
    sc.release_fx(e);
    Some(ok)
}

// ---------------------------------------------------------------------------
// Stadium and retreat

fn fast_use_stadium(ctx: &mut Ctx) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    let Ok(stadium) = turn::can_use_stadium(g, p) else { return Some(false) };
    if ctx.play_locked(stadium, &[LockedAction::UseStadium]) {
        return Some(false);
    }
    let sc = ctx.sc();
    let e = sc.new_fx(Effect::UseStadium { p: p as u8, stadium });
    let ok = crate::spec::run::use_stadium_check(sc, stadium, p, e).is_ok();
    sc.release_fx(e);
    Some(ok)
}

fn fast_retreat(ctx: &mut Ctx, bench_index: u8) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    if retreat::can_retreat(g, p, bench_index, false).is_err() {
        return Some(false);
    }
    if let Some(active) = g.st.active_pokemon(p) {
        if ctx.play_locked(active, &[LockedAction::Retreat]) {
            return Some(false);
        }
    }
    let Ok(cost) = ctx.retreat_cost() else { return Some(false) };
    if cost.is_empty() {
        return Some(true);
    }
    let active = g.st.players[p].active;
    let Ok(map) = ctx.provided(active) else { return Some(false) };
    Some(crate::energy::check_enough_energy(map.as_slice(), cost.as_slice()))
}
