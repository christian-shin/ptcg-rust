//! Legality from declared checks, not from playing each candidate on a fork (S4, "legality without trials").
//!
//! [`Ctx`] is the per-decision derived context: a facts provider (`derived::Facts`) over the game, read-only (batch 8
//! C3d: no scratch copy of the game). The derived facts (the attack list and costs, provided Energy, the Retreat Cost,
//! HP, types, the powers, "has no Ability") are computed by the derived layer's functions over it and cached for the
//! decision (the Energy a Pokémon provides is read once, whatever number of attacks and retreats ask for it); the locks,
//! permissions, restrictions and the cards' declared checks (`needs`, the rules' usability, leading `Fail` steps) read
//! through it. A played Item's checks see it out of the hand (`run::Frame::excl`), as its program will.
//!
//! [`legal_fast`] answers one action: `Some(legal)` when every rule that could reject it is declared and was
//! evaluated, `None` when one is not (the caller then plays the action on a copy, a counted fallback). It
//! calls the same `can_x()` checks the executing reducers call (`turn.rs`, `play.rs`, `retreat.rs`,
//! `attack.rs`), the same lock query (`derived::event_locked`, through the events' check functions) and the same declared card checks
//! (`spec::run`: `needs` / implied preconditions, once per turn, leading `Fail` steps), so the answer and the
//! execution can't drift apart. `PTCG_VERIFY_LEGAL=1` compares every answer with the full trial.
//!
//! What each action kind checks:
//! * Energy attach: the turn rule (`can_attach_energy`), then the Attach event the play produces
//!   (`engine::attach`): the spot, its locks (`event_locked`), the Energy card's own attach guard.
//! * Pokémon (Basic, evolve): the event the play produces (`enter::hand_play_view`), its locks
//!   (`event_locked`), and for an Evolve the evolution rules (`enter::evolve_rules`: evolves from, the
//!   rule's limits with the permissions, the restrictions).
//! * Trainer: the PlayTrainer the play produces and the checks its routine makes (`engine::play_trainer::check_with`:
//!   the turn's rules, a Tool's Attach, the locks over PlayTrainer), then the card's declared `needs` and implied
//!   preconditions evaluated in the state its effect would see.
//! * Attack: the attack list read, then the UseAttack's checks (`engine::turn_action::attack_checks`: the first turn,
//!   the Special Conditions, the lasting "can't attack", the locks over UseAttack, the cost), the leading `Fail` steps.
//! * Ability: the power list read, the core use rule, the lock probe, the locks over UseAbility, once per turn, `needs`
//!   and implied preconditions.
//! * Use Stadium: the UseStadium's checks (`engine::turn_action::stadium_checks`), the Stadium's own `needs`.
//! * Retreat: `can_retreat`, the ChangeActive's checks (`engine::change_active::check_with`: the Pokémon's lasting
//!   "can't retreat", the locks over the event), the cost against the provided Energy.
//! * Pass: always.

use crate::carddb::DefId;
use crate::effects::*;
use crate::engine::{attack, retreat, turn};
use crate::game::{Action, Game};
use crate::list::*;
use crate::prompts::get_target;
use crate::state::AttackRef;
use crate::spec::passive;
use crate::spec::run::Gate;
use crate::types::*;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Which cards declare something legality reads

const F_ATTACK_FAIL: u8 = 8;
const F_ATTACH_GUARD: u8 = 16;
const F_PLAY_SPEC: u8 = 32;
const F_PLAYS_AS_POKEMON: u8 = 64;
/// The card must be played with a companion from the hand (`CardSpec::together`): which card is the companion depends on
/// which copy is played, so its answers are not shared between copies (`Ctx::memo`).
const F_TOGETHER: u8 = 128;

fn def_flags(def: DefId) -> u8 {
    static T: OnceLock<Vec<u8>> = OnceLock::new();
    T.get_or_init(|| {
        (0..crate::carddb::cards().len())
            .map(|d| {
                let Some(spec) = crate::cards::spec_for(d as DefId) else { return 0 };
                let mut f = 0;
                for ps in spec.passives {
                    f |= match &ps.modifier {
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
                if spec.together.is_some() {
                    f |= F_TOGETHER;
                }
                f
            })
            .collect()
    })[def as usize]
}

// ---------------------------------------------------------------------------
// The per-decision context

pub struct Ctx<'a> {
    pub g: &'a Game,
    p: usize,
    /// The checked attack list and the available attacks built from it.
    attacks: Option<Result<(turn::CheckedAttacks, SVec<(AttackRef, bool), 64>), ()>>,
    /// The decision's facts: per (asking player, spot) the provided Energy, HP and types; per player the Retreat Cost; per
    /// (player, attack) the cost; per (player, card) the powers; per (player, card, power) "has no Ability".
    provided: SVec<((u8, u8, u8), crate::game::R<EnergyMap>), { 2 * crate::state::MAX_SLOT_REFS }>,
    hp: SVec<((u8, u8), crate::game::R<i32>), { crate::state::MAX_SLOT_REFS }>,
    types: SVec<((u8, u8), crate::game::R<SVec<CardType, 4>>), { crate::state::MAX_SLOT_REFS }>,
    retreat: [Option<crate::game::R<Cost>>; 2],
    attack_cost: SVec<((u8, AttackRef), crate::game::R<Cost>), 16>,
    powers: SVec<(CardId, crate::game::R<SVec<PowerRef, 8>>), 16>,
    abil: SVec<((CardId, u8), bool), 48>,
    /// The answers of the hand-card plays by (card definition, target): copies of one card in the hand share one answer
    /// (see [`legal_fast`]).
    memo: SVec<((DefId, CardTarget), bool), 96>,
    /// The reason of the last `None` from `legal_fast` (the fallback table).
    pub why: &'static str,
    /// The readers' card lists of the decision (`decl::OrderCache`).
    order: crate::decl::OrderCache,
}

impl<'a> Ctx<'a> {
    pub fn new(g: &'a Game) -> Ctx<'a> {
        Ctx {
            g,
            p: g.st.active_player as usize,
            attacks: None,
            provided: SVec::new(),
            hp: SVec::new(),
            types: SVec::new(),
            retreat: [None, None],
            attack_cost: SVec::new(),
            powers: SVec::new(),
            abil: SVec::new(),
            memo: SVec::new(),
            why: "",
            order: crate::decl::OrderCache::new(),
        }
    }

    fn none(&mut self, why: &'static str) -> Option<bool> {
        self.why = why;
        None
    }

    // --- the derived reads ---------------------------------------------------

    /// The attacks the active player may use (`derived::compute_attacks`) and the list of attack actions built from it.
    pub fn attack_list(&mut self) -> Result<&(turn::CheckedAttacks, SVec<(AttackRef, bool), 64>), ()> {
        if self.attacks.is_none() {
            let g = self.g;
            let p = self.p;
            let checked = crate::derived::Facts::attacks(self, p).map_err(|_| ());
            self.attacks = Some(checked.map(|c| {
                let all = turn::assemble_available_attacks(g, p, &c);
                (c, all)
            }));
        }
        self.attacks.as_ref().unwrap().as_ref().map_err(|_| ())
    }

    /// The Energy the active player's Pokémon in `slot` provides, read once.
    fn provided(&mut self, slot: crate::state::SlotId) -> Result<EnergyMap, ()> {
        let p = self.p;
        crate::derived::Facts::provided_energy(self, p, SlotRef::new(p, slot)).map_err(|_| ())
    }

    /// The effective Retreat Cost, read once.
    fn retreat_cost(&mut self) -> Result<Cost, ()> {
        let p = self.p;
        crate::derived::Facts::retreat_cost(self, p).map_err(|_| ())
    }

    /// Is the event forbidden by a declared lock? The one query execution makes (`passive::event_locked`), made only
    /// when a lock over events can exist (`passive::may_lock_event`).
    fn event_locked(&mut self, v: &crate::spec::event::EventView) -> bool {
        self.event_lock(v).is_some()
    }

    /// [`Ctx::event_locked`] with the lock's code (an error of a read stops the play, as in the trial).
    fn event_lock(&mut self, v: &crate::spec::event::EventView) -> Option<&'static str> {
        if v.card.is_none() || !passive::may_lock_event(self.g, v.actor() as usize, v.kind) {
            return None;
        }
        crate::spec::passive::event_locked(self, v).unwrap_or_else(|e| Some(e.0))
    }
}

/// The decision's facts (`derived::Facts`): each computed once by the derived layer's functions over the game as it is,
/// read-only (the game's own memo isn't touched: legality reads a `&Game`).
impl crate::derived::Facts for Ctx<'_> {
    #[inline]
    fn g(&self) -> &Game {
        self.g
    }
    #[inline]
    fn undelegated<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        // No copied attack's source code runs while the turn's options are listed.
        debug_assert!(self.g.deleg.is_none());
        f(self)
    }
    fn hp(&mut self, p: usize, s: crate::state::SlotId) -> crate::game::R<i32> {
        if let Some((_, r)) = self.hp.iter().find(|(k, _)| *k == (p as u8, s)) {
            return *r;
        }
        let r = crate::derived::compute_hp(self, p, s);
        self.hp.push(((p as u8, s), r));
        r
    }
    fn types(&mut self, target: SlotRef) -> crate::game::R<SVec<CardType, 4>> {
        if let Some((_, r)) = self.types.iter().find(|(k, _)| *k == (target.p, target.s)) {
            return *r;
        }
        let r = crate::derived::compute_types(self, target);
        self.types.push(((target.p, target.s), r));
        r
    }
    fn provided_energy(&mut self, p: usize, source: SlotRef) -> crate::game::R<EnergyMap> {
        let k = (p as u8, source.p, source.s);
        if let Some((_, r)) = self.provided.iter().find(|(x, _)| *x == k) {
            return *r;
        }
        let r = crate::derived::compute_provided_energy(self, p, source);
        self.provided.push((k, r));
        r
    }
    fn retreat_cost(&mut self, p: usize) -> crate::game::R<Cost> {
        if let Some(r) = self.retreat[p] {
            return r;
        }
        let r = crate::derived::compute_retreat_cost(self, p);
        self.retreat[p] = Some(r);
        r
    }
    fn attacks(&mut self, p: usize) -> crate::game::R<turn::CheckedAttacks> {
        crate::derived::compute_attacks(self, p)
    }
    fn attack_cost(&mut self, p: usize, attack: AttackRef) -> crate::game::R<Cost> {
        if let Some((_, r)) = self.attack_cost.iter().find(|(k, _)| *k == (p as u8, attack)) {
            return *r;
        }
        let r = crate::derived::compute_attack_cost(self, p, attack);
        if self.attack_cost.len() < self.attack_cost.capacity() {
            self.attack_cost.push(((p as u8, attack), r));
        }
        r
    }
    fn powers(&mut self, card: CardId) -> crate::game::R<SVec<PowerRef, 8>> {
        if let Some((_, r)) = self.powers.iter().find(|(k, _)| *k == card) {
            return *r;
        }
        let r = crate::derived::compute_powers(self, card);
        if self.powers.len() < self.powers.capacity() {
            self.powers.push((card, r));
        }
        r
    }
    fn has_no_ability(&mut self, card: CardId, power_index: Option<u8>) -> bool {
        let k = (card, power_index.unwrap_or(crate::prefabs::PROBE_GENERIC));
        if let Some((_, r)) = self.abil.iter().find(|(x, _)| *x == k) {
            return *r;
        }
        let r = crate::derived::compute_has_no_ability(self, card, power_index);
        if self.abil.len() < self.abil.capacity() {
            self.abil.push((k, r));
        }
        r
    }
    fn tools_off(&mut self) -> bool {
        crate::derived::compute_tools_off(self.g)
    }
    #[inline]
    fn decl_in_place(&mut self, f: usize, class: u16) -> SVec<CardId, 120> {
        let r = crate::decl::in_place_cached(&self.g.st, &self.g.decls, &mut self.order, Some(&self.g.order_cache), f, class);
        if crate::game::verify_cache() {
            assert_eq!(r.as_slice(), crate::decl::in_place(self.g, f, class).as_slice(), "legality's order cache differs (family {f})");
        }
        r
    }
    #[inline]
    fn decl_ordered(&mut self, set: u128, class: u16) -> SVec<CardId, 120> {
        let r = crate::decl::ordered_cached(&self.g.st, &mut self.order, Some(&self.g.order_cache), set, class);
        if crate::game::verify_cache() {
            assert_eq!(r.as_slice(), crate::decl::kind_order(self.g, set, class).as_slice(), "legality's order cache differs (class {class})");
        }
        r
    }
    #[inline]
    fn decl_kind_listeners(&mut self, kind: u32) -> SVec<CardId, 120> {
        let set = self.g.decls.kind(kind);
        let r = self.decl_ordered(set, 0);
        if crate::game::verify_cache() {
            assert_eq!(r.as_slice(), crate::decl::kind_listeners(self.g, kind).as_slice(), "legality's order cache differs (kind {kind})");
        }
        r
    }
}

/// The checks of an Attach the play produces (`engine::attach::check_attach_with`, the function execution calls):
/// the spot on the game, the locks and the card's own guard on the scratch game, each behind its plain-read gate.
impl crate::engine::attach::AttachChecks for Ctx<'_> {
    fn game(&self) -> &Game {
        self.g
    }
    fn event_locked(&mut self, v: &crate::spec::event::EventView) -> crate::game::R<Option<&'static str>> {
        Ok(self.event_lock(v))
    }
    fn guard_refuses(&mut self, v: &crate::spec::event::EventView) -> crate::game::R<bool> {
        let Some(card) = v.card else { return Ok(false) };
        if def_flags(self.g.st.cards[card as usize].def) & F_ATTACH_GUARD == 0 {
            return Ok(false);
        }
        // An error of the read stops the play, as in the trial.
        let Some(t) = v.slot else { return Ok(false) };
        Ok(passive::attach_guard_refuses(self, card, t).unwrap_or(true))
    }
}

/// The checks of the PlayTrainer a Trainer play produces (`engine::play_trainer::check_with`, the function execution
/// calls): the turn's rules on the game, the Tool's Attach and the locks on the scratch game behind their plain-read gates.
impl crate::engine::play_trainer::PlayChecks for Ctx<'_> {
    fn game(&self) -> &Game {
        self.g
    }
    fn event_locked(&mut self, v: &crate::spec::event::EventView) -> crate::game::R<Option<&'static str>> {
        Ok(self.event_lock(v))
    }
    fn tool_attach(&mut self, card: CardId, target: SlotRef) -> crate::game::R<Option<&'static str>> {
        let v = crate::engine::attach::attach_view(self.g, card, target, crate::spec::event::RulesZone::Hand, false, crate::cause::Cause::rule(crate::cause::RuleWhich::Action, self.p as u8));
        crate::engine::attach::check_attach_with(self, &v)
    }
    fn permitted(&mut self, v: &crate::spec::event::EventView, limit: crate::spec::event::Limit) -> crate::game::R<bool> {
        crate::engine::enter::permitted(self, v, limit)
    }
}

/// The checks of a turn action (`engine::turn_action`: UseAttack, UseAbility, UseStadium), the functions execution calls:
/// the plain reads on the game, the locks and the Energy reads on the scratch game behind their gates.
impl crate::engine::turn_action::ActionChecks for Ctx<'_> {
    fn game(&self) -> &Game {
        self.g
    }
    fn event_locked(&mut self, v: &crate::spec::event::EventView) -> crate::game::R<Option<&'static str>> {
        Ok(self.event_lock(v))
    }
    fn energy_count(&mut self, slot: SlotRef) -> crate::game::R<i32> {
        match self.provided(slot.s) {
            Ok(map) => Ok(attack::max_energy_count(&map)),
            Err(()) => Err(crate::game::GameError("ENERGY_READ")),
        }
    }
    fn payable(&mut self, attack_ref: AttackRef, slot: SlotRef) -> crate::game::R<bool> {
        let p = self.p;
        let Ok(cost) = crate::derived::Facts::attack_cost(self, p, attack_ref) else { return Ok(false) };
        let Ok(map) = self.provided(slot.s) else { return Ok(false) };
        Ok(crate::energy::check_enough_energy(map.as_slice(), cost.as_slice()))
    }
    fn ability_gone(&mut self, power: crate::effects::PowerRef, card: CardId) -> bool {
        debug_assert_eq!(power.card, card);
        crate::derived::Facts::has_no_ability(self, card, Some(power.index))
    }
}

/// The checks of a ChangeActive the retreat produces (`engine::change_active::check_with`, the function execution
/// calls): the spots and the Pokémon's lasting "can't retreat" on the game, the locks on the scratch game behind their
/// plain-read gate. The retreat is a turn action, so one execution would refuse is illegal. Its cause is a rule, which
/// no `Prevent` declaration over ChangeActive can match (each needs an attack or an Ability: unit test
/// `change_active::tests::no_prevention_matches_a_rule_cause`), so the declared preventions are not read here (reading
/// them would cost a scratch game per decision); nor are the lasting ones an attack leaves on a Pokémon
/// (the lasting table's preventions, `engine::lasting`), which all name the opponent's attacks.
impl crate::engine::change_active::ActiveChecks for Ctx<'_> {
    fn game(&self) -> &Game {
        self.g
    }
    fn event_locked(&mut self, v: &crate::spec::event::EventView) -> crate::game::R<Option<&'static str>> {
        Ok(self.event_lock(v))
    }
    fn event_prevented(&mut self, v: &crate::spec::event::EventView) -> crate::game::R<bool> {
        debug_assert!(matches!(v.cause.kind, crate::cause::CauseKind::Rule { .. }), "legality checks the retreat only");
        Ok(false)
    }
}

/// Answer one action from the declared checks; `None` (with the reason in `ctx.why`) when it can't.
///
/// The play of a hand card is answered once per (card definition, target) in a decision: the checks of a play read the
/// card's definition and zone (the hand), the target and the game, never which copy it is, so two copies of one card in
/// the hand get one answer (the unit test `copies_in_the_hand_answer_alike` plays every pool card's twin;
/// `PTCG_VERIFY_LEGAL=1` compares every memo answer with a fresh one). A card played with a companion
/// (`F_TOGETHER`) is left out: its companion is the first other matching card, which depends on the copy.
pub fn legal_fast(ctx: &mut Ctx, a: Action) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    match a {
        Action::Pass => Some(true),
        Action::PlayCard { hand_index, target } => {
            let Some(card) = g.st.players[p].hand.get(hand_index as usize) else { return ctx.none("unknown hand card") };
            let def = d_def(g, card);
            if def_flags(def) & F_TOGETHER != 0 {
                return play_fresh(ctx, card, target);
            }
            if let Some(&(_, answer)) = ctx.memo.iter().find(|(k, _)| *k == (def, target)) {
                if crate::options::verify_legal() {
                    assert_eq!(play_fresh(ctx, card, target), Some(answer), "the memoized play answer differs from the fresh one: {} onto {:?}", g.st.cdef(card).full_name, target);
                }
                return Some(answer);
            }
            let r = play_fresh(ctx, card, target);
            if let Some(answer) = r {
                if ctx.memo.len() < ctx.memo.capacity() {
                    ctx.memo.push(((def, target), answer));
                }
            }
            r
        }
        Action::Attack { name, from } => fast_attack(ctx, name, from),
        Action::UseAbility { name, target } => fast_ability(ctx, name, target),
        Action::UseStadium => fast_use_stadium(ctx),
        Action::Retreat { bench_index } => fast_retreat(ctx, bench_index),
    }
}

/// The play of hand card `card` onto `target`, from the checks (no memo).
fn play_fresh(ctx: &mut Ctx, card: CardId, target: CardTarget) -> Option<bool> {
    let d = ctx.g.st.cdef(card);
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

// ---------------------------------------------------------------------------
// Energy

fn fast_energy(ctx: &mut Ctx, card: CardId, target: CardTarget) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    let Ok(t) = turn::can_attach_energy(g, p, target) else { return Some(false) };
    // No lock over Attach can exist and the card has no guard of its own: the spot is the only check left (two plain
    // reads), so the event isn't built.
    if def_flags(d_def(g, card)) & F_ATTACH_GUARD == 0 && !passive::may_lock_event(g, p, crate::spec::event::EventKind::Attach) {
        return Some(crate::engine::attach::attach_spot_ok(g, card, t, g.st.owner(card) as u8).is_ok());
    }
    // The Attach the play produces and the checks its routine makes (`engine::attach::check_attach_with`): the
    // spot, the locks, the Energy's own guard.
    let v = crate::engine::attach::attach_view(g, card, t, crate::spec::event::RulesZone::Hand, true, crate::cause::Cause::rule(crate::cause::RuleWhich::Action, p as u8));
    Some(matches!(crate::engine::attach::check_attach_with(ctx, &v), Ok(None)))
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
            return Some(matches!(crate::engine::enter::restricted(ctx, &v), Ok(None)));
        }
        return Some(true);
    }
    let reach = crate::engine::enter::Reach::Next;
    match crate::engine::enter::evolve_rules_fast(g, &v, reach) {
        Some(ok) => Some(ok),
        None => Some(crate::engine::enter::evolve_rules(ctx, &v, reach).is_ok()),
    }
}

// ---------------------------------------------------------------------------
// Trainers

fn fast_trainer(ctx: &mut Ctx, card: CardId, target: CardTarget) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    let d = g.st.cdef(card);
    // The PlayTrainer the play produces and the checks its routine makes (the turn's rules, a Tool's Attach, the locks).
    let t = get_target(&g.st, p, target).ok();
    let cause = crate::cause::Cause::rule(crate::cause::RuleWhich::Action, p as u8);
    let v = crate::engine::play_trainer::play_view(g, card, crate::spec::event::TrainerUse::Played, crate::spec::event::RulesZone::Hand, cause);
    if !matches!(crate::engine::play_trainer::check_with(ctx, &v, t), Ok(None)) {
        return Some(false);
    }
    match d.trainer_type() {
        TrainerType::Item => {
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
                        if ctx.event_locked(&v) || (crate::engine::enter::may_be_restricted(g, &v) && !matches!(crate::engine::enter::restricted(ctx, &v), Ok(None))) {
                            return Some(false);
                        }
                    }
                }
            }
            // The Item (and its companion) have left the hand for the play area when its effect runs: its checks see them
            // out of the hand (`run::Frame::excl`).
            let with = crate::engine::play_trainer::companion(g, p, card);
            let _ = cause;
            Some(crate::spec::run::trainer_play_check(ctx, card, p, [card, with.unwrap_or(NO_CARD)]).is_ok())
        }
        TrainerType::Supporter => {
            if def_flags(d_def(g, card)) & F_PLAY_SPEC == 0 {
                return Some(true);
            }
            Some(crate::spec::run::trainer_play_check(ctx, card, p, [NO_CARD; 2]).is_ok())
        }
        TrainerType::Stadium => Some(true),
        TrainerType::Tool => {
            // No Tool declares a play spec: its effects are passives (a Tool with one would need the state
            // after it is attached).
            if def_flags(d_def(g, card)) & F_PLAY_SPEC != 0 {
                return ctx.none("tool with a play spec");
            }
            Some(true)
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
    // The first-turn permission: the attack's own text, or an Ability (Meloetta ex).
    let granted = passive::grants_first_turn_attack(ctx, p, attack);
    // The UseAttack's checks (`engine::turn_action::attack_checks`, the function execution calls).
    let attacking = match crate::engine::turn_action::attack_checks(ctx, p, attack, false, granted) {
        Ok(Ok(s)) => s,
        _ => return Some(false),
    };
    // The preconditions the attack's text declares (its leading `Fail` steps).
    if def_flags(d_def(g, attack.card)) & F_ATTACK_FAIL != 0 {
        // (An attack copied from a Benched Pokémon runs its text as the copycat's.)
        let me = if copied { g.st.slot_pokemon(p, attacking.s).unwrap_or(attack.card) } else { attack.card };
        let gate = crate::spec::run::attack_gate(ctx, me, attack, p);
        match gate {
            Ok(Gate::Open) => {}
            Ok(Gate::Closed) | Err(_) => return Some(false),
            Ok(Gate::Undeclared) => return ctx.none("attack: a Fail step after other steps"),
        }
    }
    Some(true)
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
    if crate::spec::run::power_once_check(g, card, power.index, p).is_err() {
        return Some(false);
    }
    // The powers the card has now (in a game with an Ability lock).
    if g.kinds_present.has(k::DECLARES_ABILITY_LOCK) {
        match crate::derived::Facts::powers(ctx, card) {
            Ok(after) => match after.iter().find(|r| g.st.cdef(r.card).powers[r.index as usize].name == name) {
                Some(r) => power = *r,
                None => return Some(false),
            },
            Err(_) => return Some(false),
        }
    }
    // The UseAbility event's checks (`engine::turn_action::ability_checks`, the function execution calls).
    if !matches!(crate::engine::turn_action::ability_checks(ctx, p, power, card, target.slot), Ok(None)) {
        return Some(false);
    }
    Some(crate::spec::run::power_check(ctx, card, power.index, p).is_ok())
}

// ---------------------------------------------------------------------------
// Stadium and retreat

fn fast_use_stadium(ctx: &mut Ctx) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    let _ = g;
    let stadium = match crate::engine::turn_action::stadium_checks(ctx, p) {
        Ok(Ok(s)) => s,
        _ => return Some(false),
    };
    Some(crate::spec::run::use_stadium_check(ctx, stadium, p).is_ok())
}

fn fast_retreat(ctx: &mut Ctx, bench_index: u8) -> Option<bool> {
    let g = ctx.g;
    let p = ctx.p;
    if retreat::can_retreat(g, p, bench_index, false).is_err() {
        return Some(false);
    }
    // The ChangeActive the retreat produces, with the checks its routine makes.
    let Some(c) = retreat::retreat_change(g, p, bench_index) else { return Some(false) };
    let v = crate::engine::change_active::view(g, &c);
    if !matches!(crate::engine::change_active::check_with(ctx, &v), Ok(None)) {
        return Some(false);
    }
    let Ok(cost) = ctx.retreat_cost() else { return Some(false) };
    if cost.is_empty() {
        return Some(true);
    }
    let active = g.st.players[p].active;
    let Ok(map) = ctx.provided(active) else { return Some(false) };
    Some(crate::energy::check_enough_energy(map.as_slice(), cost.as_slice()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const BASIC: &str = "Duraludon PRE 69";
    const OTHER: &str = "Team Rocket's Mewtwo ex DRI 81";
    const ENERGY: &str = "Metal Energy MEE 8";

    /// A game where player 0 holds two copies of `def` in the hand, with a Pokémon and a Benched one in play.
    fn twins(def: DefId, energy_played: bool) -> Option<Game> {
        let id = |n: &str| crate::carddb::def_by_full_name(n).unwrap();
        let mut deck: Vec<DefId> = Vec::new();
        deck.extend([def; 4]);
        deck.extend([id(BASIC); 4]);
        deck.extend([id(OTHER); 2]);
        deck.resize(60, id(ENERGY));
        for seed in 1..8 {
            let mut g = Game::new(seed);
            if g.start([&deck, &deck]).is_err() {
                return None;
            }
            g.settle().ok();
            if g.st.active_player != 0 {
                continue;
            }
            let name = crate::carddb::def(def).full_name;
            let sc = json!({
                "me": {"reset": true, "active": BASIC, "active_energy": [ENERGY], "bench": [{"card": OTHER}], "hand": [name, name]},
                "opp": {"reset": true, "active": BASIC}
            });
            crate::scenario::apply(&mut g, &sc).ok()?;
            if energy_played {
                g.st.players[0].energy_played_turn = g.st.turn;
            }
            return Some(g);
        }
        None
    }

    /// The memo of the hand-card plays by (definition, target) is valid because a play's checks read the card's definition and
    /// zone, never which copy it is: two copies in the hand get the same answer onto every target, for every card of the pool
    /// that isn't played with a companion (`F_TOGETHER`: its companion depends on the copy).
    #[test]
    fn copies_in_the_hand_answer_alike() {
        let mut checked = 0;
        for def in 0..crate::carddb::cards().len() as DefId {
            if crate::carddb::def(def).full_name.is_empty() || def_flags(def) & F_TOGETHER != 0 {
                continue;
            }
            for energy_played in [false, true] {
                let Some(g) = twins(def, energy_played) else { continue };
                let hand = g.st.players[0].hand;
                if hand.len() != 2 || d_def(&g, hand.get(0).unwrap()) != def {
                    continue;
                }
                for a in crate::options::candidate_actions(&g) {
                    let Action::PlayCard { hand_index: 0, target } = a else { continue };
                    let (c0, c1) = (hand.get(0).unwrap(), hand.get(1).unwrap());
                    let first = play_fresh(&mut Ctx::new(&g), c0, target);
                    let second = play_fresh(&mut Ctx::new(&g), c1, target);
                    assert_eq!(first, second, "{} onto {:?}: the two copies in the hand answer differently", g.st.cdef(c0).full_name, target);
                    checked += 1;
                }
            }
        }
        assert!(checked > 1000, "only {checked} twin plays were compared");
    }

    /// The cards played with a companion are exactly those with `CardSpec::together`.
    #[test]
    fn the_companion_flag_is_the_spec_field() {
        for def in 0..crate::carddb::cards().len() as DefId {
            let together = crate::cards::spec_for(def).map_or(false, |s| s.together.is_some());
            assert_eq!(def_flags(def) & F_TOGETHER != 0, together, "{}", crate::carddb::def(def).full_name);
        }
    }
}
