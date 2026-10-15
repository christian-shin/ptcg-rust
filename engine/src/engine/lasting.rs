//! The lasting table (events batch 8 C7; plan section 5.3; events design 4.4: "lasting effects become declarations stored
//! with a timing"): every effect an attack leaves on a Pokémon or a player for a time is one entry of `State::lasting`
//! ([`LastingEffect`]): the locks (APR C-15, C-19), the preventions, the damage and cost changes, "no Weakness", the
//! next-turn bonus, and the two lasting triggers (a delayed trap's Retaliate; the discard of the Attacking Pokémon's Energy when this Pokémon is Knocked Out). The ApplyEffect event's reducer puts them
//! (`engine::apply`); the readers here answer what is in force; the end of each turn rolls their timing over
//! ([`end_turn`], [`after_end_turn`]); a Pokémon's go with its other effects ([`clear_effects`],
//! [`remove_attack_effects`], [`clear_spot`]; APR C-13, C-15).
//!
//! Timing (unchanged from the slot fields it replaces): an effect put during its attacker's turn is "pending" until the
//! end of that turn, in force during the opponent's next turn, and gone at the end of it; a lock counts the turns of the
//! locked player.

use crate::list::{CardId, CardList, SVec};
use crate::state::{AttackRef, LastingEffect, LastingOn, LastingWhat, SlotId, State};

/// Which kind of entry (one entry per Pokémon for each kind but the locks and the preventions).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    TakesLess,
    DealsLess,
    TakesMore,
    RetaliatePending,
    RetaliateArmed,
    DiscardEnergy,
    AttackCost,
    RetreatCost,
    NoWeakness,
    BonusPending,
    BonusArmed,
}

fn kind_of(w: &LastingWhat) -> Option<Kind> {
    Some(match *w {
        LastingWhat::TakesLessDamage(_) => Kind::TakesLess,
        LastingWhat::DealsLessDamage(_) => Kind::DealsLess,
        LastingWhat::TakesMoreDamage { .. } => Kind::TakesMore,
        LastingWhat::Retaliate { pending: true, .. } => Kind::RetaliatePending,
        LastingWhat::Retaliate { pending: false, .. } => Kind::RetaliateArmed,
        LastingWhat::DiscardAttackerEnergy { .. } => Kind::DiscardEnergy,
        LastingWhat::AttackCostIncrease { .. } => Kind::AttackCost,
        LastingWhat::RetreatCostIncrease { .. } => Kind::RetreatCost,
        LastingWhat::NoWeakness { .. } => Kind::NoWeakness,
        LastingWhat::NextTurnBonus { pending: true, .. } => Kind::BonusPending,
        LastingWhat::NextTurnBonus { pending: false, .. } => Kind::BonusArmed,
        LastingWhat::Lock { .. } | LastingWhat::PlayerLock { .. } | LastingWhat::Prevent { .. } | LastingWhat::Spawn { .. } | LastingWhat::Declared { .. } | LastingWhat::Recorded => return None,
    })
}

#[inline]
fn find(st: &State, on: LastingOn, k: Kind) -> Option<&LastingEffect> {
    st.lasting.iter().find(|l| l.on == on && kind_of(&l.what) == Some(k))
}

/// Replace the entry of kind `k` on `on` with `e` (`None`: remove it); a spent one is removed.
fn set(st: &mut State, on: LastingOn, k: Kind, e: Option<LastingEffect>) {
    let e = e.filter(|e| !e.what.is_spent());
    match st.lasting.iter().position(|l| l.on == on && kind_of(&l.what) == Some(k)) {
        Some(i) => match e {
            Some(e) => st.lasting.as_mut_slice()[i] = e,
            None => {
                st.lasting.remove_at(i);
            }
        },
        None => {
            if let Some(e) = e {
                st.lasting.push(e);
            }
        }
    }
}

/// Change the entry of kind `k` on `on` (made from `blank` when there is none); a spent one is removed.
fn update(st: &mut State, on: LastingOn, k: Kind, blank: LastingWhat, source: CardId, f: impl FnOnce(&mut LastingEffect)) {
    let mut e = find(st, on, k).copied().unwrap_or(LastingEffect { on, source, what: blank });
    f(&mut e);
    set(st, on, k, Some(e));
}

/// Change the entry of kind `k` on `on`, if there is one; a spent one is removed.
fn modify(st: &mut State, on: LastingOn, k: Kind, f: impl FnOnce(&mut LastingEffect)) {
    if let Some(mut e) = find(st, on, k).copied() {
        f(&mut e);
        set(st, on, k, Some(e));
    }
}

/// The spots player `p` has in play (with cards), as the end-of-turn processing visits them.
fn in_play(st: &State, p: usize) -> crate::state::InPlay {
    st.players[p].in_play()
}

// ---------------------------------------------------------------------------
// The readers

/// "Takes N less damage from attacks" on the Pokémon in spot `s` of player `p`: every one on it, added up (APR B-05: "If
/// several instances of 5. Effects on the Pokémon taking the damage apply at the same time, they must all be included";
/// JP FAQ Iron Defender: two copies, 60 less).
pub fn takes_less_damage(st: &State, p: usize, s: SlotId) -> i32 {
    let on = LastingOn::pokemon(p, s);
    st.lasting.iter().filter(|l| l.on == on).map(|l| if let LastingWhat::TakesLessDamage(n) = l.what { n as i32 } else { 0 }).sum()
}

/// "Attacks used by the Defending Pokémon do N less damage" on the Pokémon in spot `s` of player `p`.
pub fn deals_less_damage(st: &State, p: usize, s: SlotId) -> i32 {
    match find(st, LastingOn::pokemon(p, s), Kind::DealsLess) {
        Some(LastingEffect { what: LastingWhat::DealsLessDamage(n), .. }) => *n as i32,
        _ => 0,
    }
}

/// "The Defending Pokémon takes N more damage" in force on the Pokémon in spot `s` of player `p` against player
/// `attacker`'s attacks (put during that player's previous turn: no longer pending).
pub fn takes_more_damage(st: &State, p: usize, s: SlotId, attacker: u8) -> i32 {
    match find(st, LastingOn::pokemon(p, s), Kind::TakesMore) {
        Some(LastingEffect { what: LastingWhat::TakesMoreDamage { amount, attacker: a, pending: false, .. }, .. }) if *amount > 0 && *a == Some(attacker) => *amount as i32,
        _ => 0,
    }
}

/// "This Pokémon has no Weakness" in force on the Pokémon in spot `s` of player `p`.
pub fn no_weakness(st: &State, p: usize, s: SlotId) -> bool {
    matches!(find(st, LastingOn::pokemon(p, s), Kind::NoWeakness), Some(LastingEffect { what: LastingWhat::NoWeakness { armed: true, .. }, .. }))
}

/// The lasting "[C] more" on the attacks of the Pokémon in spot `s` of player `p`.
pub fn attack_cost_increase(st: &State, p: usize, s: SlotId) -> i32 {
    match find(st, LastingOn::pokemon(p, s), Kind::AttackCost) {
        Some(LastingEffect { what: LastingWhat::AttackCostIncrease { now, .. }, .. }) => *now as i32,
        _ => 0,
    }
}

/// The lasting "[C] more" on the Retreat Cost of the Pokémon in spot `s` of player `p`.
pub fn retreat_cost_increase(st: &State, p: usize, s: SlotId) -> i32 {
    match find(st, LastingOn::pokemon(p, s), Kind::RetreatCost) {
        Some(LastingEffect { what: LastingWhat::RetreatCostIncrease { now, .. }, .. }) => *now as i32,
        _ => 0,
    }
}

/// The armed Retaliate trigger on the Pokémon in spot `s` of player `p` (none while one put this turn is pending: the
/// newer replaces it at the end of the turn).
pub fn armed_retaliate(st: &State, p: usize, s: SlotId) -> Option<LastingEffect> {
    let on = LastingOn::pokemon(p, s);
    if find(st, on, Kind::RetaliatePending).is_some() {
        return None;
    }
    find(st, on, Kind::RetaliateArmed).copied()
}

/// The armed "when this Pokémon is Knocked Out, discard an Energy from the Attacking Pokémon" trigger on the Pokémon in spot `s` of player `p`: (its attack, the attacking player, the
/// source card), when armed and not pending.
pub fn armed_discard_attacker_energy(st: &State, p: usize, s: SlotId) -> Option<(AttackRef, u8, CardId)> {
    match find(st, LastingOn::pokemon(p, s), Kind::DiscardEnergy) {
        Some(LastingEffect { source, what: LastingWhat::DiscardAttackerEnergy { attack, attacker, armed: true, pending: false }, .. }) => Some((*attack, *attacker, *source)),
        _ => None,
    }
}

/// The armed next-turn bonus on the Pokémon in spot `s` of player `p`: its declaration, the card it is for and the attack.
pub fn armed_bonus(st: &State, p: usize, s: SlotId) -> Option<(&'static crate::spec::passive::NextTurnBonusSpec, CardId, AttackRef)> {
    match find(st, LastingOn::pokemon(p, s), Kind::BonusArmed) {
        Some(LastingEffect { source, what: LastingWhat::NextTurnBonus { spec, attack, .. }, .. }) => Some((*spec, *source, *attack)),
        _ => None,
    }
}

/// The locks on the Pokémon in spot `s` of player `p`, in the order they were put.
pub fn pokemon_locks(st: &State, p: usize, s: SlotId) -> SVec<LastingEffect, 4> {
    let on = LastingOn::pokemon(p, s);
    let mut out = SVec::new();
    for l in st.lasting.iter() {
        if l.on == on && matches!(l.what, LastingWhat::Lock { .. }) {
            out.push(*l);
        }
    }
    out
}

/// The locks on player `p`, in their order.
pub fn player_locks(st: &State, p: usize) -> SVec<LastingEffect, 6> {
    let on = LastingOn::Player(p as u8);
    let mut out = SVec::new();
    for l in st.lasting.iter() {
        if l.on == on && matches!(l.what, LastingWhat::PlayerLock { .. }) {
            out.push(*l);
        }
    }
    out
}

/// Can a lock an attack left forbid something of player `p`: one on the player that forbids anything, or one on a
/// Pokémon of theirs?
pub fn any_lock_of(st: &State, p: usize) -> bool {
    st.lasting.iter().any(|l| match (l.on, l.what) {
        (LastingOn::Player(q), LastingWhat::PlayerLock { decl, .. }) => q as usize == p && !decl.forbids.is_never(),
        (LastingOn::Pokemon(q, _), LastingWhat::Lock { .. }) => q as usize == p,
        _ => false,
    })
}

/// The preventions on the Pokémon in spot `s` of player `p`, in the order they were put.
pub fn prevents(st: &State, p: usize, s: SlotId) -> SVec<LastingEffect, 8> {
    let on = LastingOn::pokemon(p, s);
    let mut out = SVec::new();
    for l in st.lasting.iter() {
        if l.on == on && matches!(l.what, LastingWhat::Prevent { .. }) {
            out.push(*l);
        }
    }
    out
}

/// The name of the attack a Pokémon lock names.
pub fn lock_attack_name(st: &State, a: AttackRef) -> &'static str {
    st.cdef(a.card).attacks[a.idx()].name
}

// ---------------------------------------------------------------------------
// Putting them (the ApplyEffect reducer)

/// A lock on the Pokémon; the same declaration naming the same attack already there keeps the longer; at most 4.
pub fn add_pokemon_lock(st: &mut State, p: usize, s: SlotId, decl: &'static crate::spec::passive::LockDecl, turns: i8, source: CardId, attack: Option<AttackRef>) {
    let on = LastingOn::pokemon(p, s);
    let name = attack.map(|a| lock_attack_name(st, a));
    let mut n = 0;
    for i in 0..st.lasting.len() {
        let l = st.lasting.as_slice()[i];
        if l.on != on {
            continue;
        }
        if let LastingWhat::Lock { decl: d, turns: t, attack: a } = l.what {
            n += 1;
            if d.same_as(decl) && a.map(|a| lock_attack_name(st, a)) == name {
                st.lasting.as_mut_slice()[i].what = LastingWhat::Lock { decl: d, turns: t.max(turns), attack: a };
                return;
            }
        }
    }
    if n < 4 {
        st.lasting.push(LastingEffect { on, source, what: LastingWhat::Lock { decl, turns, attack } });
    }
}

/// A lock on player `q` for `turns` of their turns (a lock already there, the same declaration, keeps the longer); at
/// most 6, the last replaced when full; read in the order of their places.
pub fn add_player_lock(st: &mut State, q: usize, decl: &'static crate::spec::passive::LockDecl, turns: i8, source: CardId) {
    let turns = turns.max(1);
    let on = LastingOn::Player(q as u8);
    for l in st.lasting.as_mut_slice().iter_mut() {
        if l.on != on {
            continue;
        }
        if let LastingWhat::PlayerLock { decl: d, turns: t, pos } = l.what {
            if d.same_as(decl) {
                l.what = LastingWhat::PlayerLock { decl: d, turns: t.max(turns), pos };
                return;
            }
        }
    }
    let mut used = [false; 6];
    for l in st.lasting.iter() {
        if let (true, LastingWhat::PlayerLock { pos, .. }) = (l.on == on, l.what) {
            used[pos as usize] = true;
        }
    }
    let pos = used.iter().position(|u| !u).unwrap_or(5) as u8;
    st.lasting.retain(|l| !(l.on == on && matches!(l.what, LastingWhat::PlayerLock { pos: x, .. } if x == pos)));
    // In the order of the places: before the first of the player's with a later place.
    let at = st.lasting.iter().position(|l| l.on == on && matches!(l.what, LastingWhat::PlayerLock { pos: x, .. } if x > pos)).unwrap_or(st.lasting.len());
    st.lasting.insert(at, LastingEffect { on, source, what: LastingWhat::PlayerLock { decl, turns, pos } });
}

/// A prevention on the Pokémon, pending until the end of this turn; a newer one of the same declaration replaces the
/// older (at the end of the list).
pub fn add_prevent(st: &mut State, p: usize, s: SlotId, spec: u8, source: CardId) {
    let on = LastingOn::pokemon(p, s);
    st.lasting.retain(|l| !(l.on == on && matches!(l.what, LastingWhat::Prevent { spec: x, .. } if x == spec)));
    st.lasting.push(LastingEffect { on, source, what: LastingWhat::Prevent { spec, pending: true } });
}

/// "Takes N less damage": one entry per source card (the same card's again replaces it; different ones add up).
pub fn set_takes_less(st: &mut State, p: usize, s: SlotId, n: i32, source: CardId) {
    let on = LastingOn::pokemon(p, s);
    st.lasting.retain(|l| !(l.on == on && l.source == source && matches!(l.what, LastingWhat::TakesLessDamage(_))));
    st.lasting.push(LastingEffect { on, source, what: LastingWhat::TakesLessDamage(n as i16) });
}

/// `Op::ArmEach`'s entry on player `q`: the effect for the Pokémon of theirs that come into play while it lasts.
pub fn add_spawn(st: &mut State, q: usize, spec: &'static crate::spec::ops::state::ArmEachSpec, source: CardId, cause: crate::cause::Cause, attack: Option<AttackRef>, ends_with: u8) {
    let _ = attack;
    st.lasting.push(LastingEffect { on: LastingOn::Player(q as u8), source, what: LastingWhat::Spawn { spec, cause: cause.pack(), ends_with } });
}

/// The `Op::ArmEach` entries on player `q` (the effects a new Pokémon of theirs gets).
pub fn spawns(st: &State, q: usize) -> SVec<LastingEffect, 4> {
    let on = LastingOn::Player(q as u8);
    let mut out = SVec::new();
    for l in st.lasting.iter() {
        if l.on == on && matches!(l.what, LastingWhat::Spawn { .. }) && out.len() < out.capacity() {
            out.push(*l);
        }
    }
    out
}

/// "Attacks used by the Defending Pokémon do N less damage".
pub fn set_deals_less(st: &mut State, p: usize, s: SlotId, n: i32, source: CardId) {
    let on = LastingOn::pokemon(p, s);
    set(st, on, Kind::DealsLess, Some(LastingEffect { on, source, what: LastingWhat::DealsLessDamage(n.max(0) as i16) }));
}

/// "The Defending Pokémon takes N more damage" by player `attacker`'s attacks: put pending, or, when one of the same
/// attacker's is in force, re-armed after the attack in progress.
pub fn set_takes_more(st: &mut State, p: usize, s: SlotId, n: i32, attacker: u8, source: CardId) {
    let blank = LastingWhat::TakesMoreDamage { amount: 0, attacker: None, pending: false, rearm: false };
    update(st, LastingOn::pokemon(p, s), Kind::TakesMore, blank, source, |e| {
        if let LastingWhat::TakesMoreDamage { amount, attacker: a, pending, rearm } = &mut e.what {
            let already = *amount > 0 && !*pending && *a == Some(attacker);
            *amount = n as i16;
            *a = Some(attacker);
            if already {
                *rearm = true;
            } else {
                *pending = true;
            }
        }
    });
}

/// The TakesMore damage re-armed after an attack (the AfterAttack reducer): `rearm` becomes `pending`.
pub fn rearm_takes_more_after_attack(st: &mut State, o: usize) {
    for s in in_play(st, o) {
        let on = LastingOn::pokemon(o, s);
        if let Some(LastingEffect { what: LastingWhat::TakesMoreDamage { rearm: true, .. }, .. }) = find(st, on, Kind::TakesMore) {
            modify(st, on, Kind::TakesMore, |e| {
                if let LastingWhat::TakesMoreDamage { pending, rearm, .. } = &mut e.what {
                    *rearm = false;
                    *pending = true;
                }
            });
        }
    }
}

/// A Retaliate trigger put this turn (pending).
pub fn set_retaliate(st: &mut State, p: usize, s: SlotId, amount: i32, attack: AttackRef, attacker: u8, source: CardId) {
    let on = LastingOn::pokemon(p, s);
    set(st, on, Kind::RetaliatePending, Some(LastingEffect { on, source, what: LastingWhat::Retaliate { amount: amount as i16, attack, attacker, pending: true } }));
}

/// The "when this Pokémon is Knocked Out, discard an Energy from the Attacking Pokémon" trigger put this turn.
pub fn set_discard_attacker_energy(st: &mut State, p: usize, s: SlotId, attack: AttackRef, attacker: u8, source: CardId) {
    let on = LastingOn::pokemon(p, s);
    set(st, on, Kind::DiscardEnergy, Some(LastingEffect { on, source, what: LastingWhat::DiscardAttackerEnergy { attack, attacker, armed: true, pending: true } }));
}

/// "Costs [C] more" put this turn by player `attacker` (`retreat`: the Retreat Cost).
pub fn set_cost_increase(st: &mut State, p: usize, s: SlotId, retreat: bool, attacker: u8, source: CardId) {
    let (k, blank) = if retreat {
        (Kind::RetreatCost, LastingWhat::RetreatCostIncrease { now: 0, pending: 0, attacker: None })
    } else {
        (Kind::AttackCost, LastingWhat::AttackCostIncrease { now: 0, pending: 0, attacker: None })
    };
    update(st, LastingOn::pokemon(p, s), k, blank, source, |e| {
        if let LastingWhat::AttackCostIncrease { pending, attacker: a, .. } | LastingWhat::RetreatCostIncrease { pending, attacker: a, .. } = &mut e.what {
            *pending = 1;
            *a = Some(attacker);
        }
    });
}

/// "No Weakness" put this turn.
pub fn set_no_weakness(st: &mut State, p: usize, s: SlotId, source: CardId) {
    update(st, LastingOn::pokemon(p, s), Kind::NoWeakness, LastingWhat::NoWeakness { armed: false, pending: false }, source, |e| {
        if let LastingWhat::NoWeakness { pending, .. } = &mut e.what {
            *pending = true;
        }
    });
}

/// The next-turn bonus armed this turn (pending) for `source`.
pub fn set_bonus_pending(st: &mut State, p: usize, s: SlotId, spec: &'static crate::spec::passive::NextTurnBonusSpec, source: CardId, attack: AttackRef) {
    let on = LastingOn::pokemon(p, s);
    set(st, on, Kind::BonusPending, Some(LastingEffect { on, source, what: LastingWhat::NextTurnBonus { spec, attack, pending: true } }));
}

// ---------------------------------------------------------------------------
// Ending them

/// The Pokémon's effects end ("PokemonCardList.clearEffects()" for the lasting ones: moving to the Bench, leaving play):
/// its locks, damage changes, "no Weakness", the Knock Out trigger, the TakesMore damage, Retaliate and the preventions; not the
/// cost increases nor the next-turn bonus (the attacking side's).
pub fn clear_effects(st: &mut State, p: usize, s: SlotId) {
    clear_effects_keeping(st, p, s, true);
}

/// The Pokémon evolves or is devolved: every effect on it ends, a Trainer's too (an effect on the Pokémon, APR B-05 and the
/// examples of "effects on a Pokémon"; C-13; official JP FAQ Acerola's Mischief on a devolved Pokémon); not the cost
/// increases nor the next-turn bonus nor the player's.
pub fn clear_effects_evolving(st: &mut State, p: usize, s: SlotId) {
    clear_effects_keeping(st, p, s, false);
}

fn clear_effects_keeping(st: &mut State, p: usize, s: SlotId, keep_trainers: bool) {
    let on = LastingOn::pokemon(p, s);
    let by_trainer = |st: &State, c: CardId| st.cdef(c).is_trainer();
    let keep: Vec<bool> = st
        .lasting
        .iter()
        .map(|l| {
            l.on != on
                || matches!(l.what, LastingWhat::AttackCostIncrease { .. } | LastingWhat::RetreatCostIncrease { .. } | LastingWhat::NextTurnBonus { .. } | LastingWhat::PlayerLock { .. } | LastingWhat::Declared { .. } | LastingWhat::Recorded)
                // A Trainer's effect on the Pokémon (Iron Defender) isn't an effect of an attack: it stays on the Bench move (APR A-05, C-15); evolving ends it ([`clear_effects_evolving`]).
                || (keep_trainers && by_trainer(st, l.source))
        })
        .collect();
    let mut i = 0;
    st.lasting.retain(|_| {
        i += 1;
        keep[i - 1]
    });
}

/// The effects of attacks on the Pokémon end: every lasting effect of an attack on it.
pub fn remove_attack_effects(st: &mut State, p: usize, s: SlotId) {
    let on = LastingOn::pokemon(p, s);
    let keep: Vec<bool> = st.lasting.iter().map(|l| l.on != on || st.cdef(l.source).is_trainer() || matches!(l.what, LastingWhat::Declared { .. } | LastingWhat::Recorded)).collect();
    let mut i = 0;
    st.lasting.retain(|_| {
        i += 1;
        keep[i - 1]
    });
}

/// The effects cards declared and put on the Pokémon in spot `s` of player `p`, and the records cards made of it, end with
/// the Pokémon's other effects when it evolves or is devolved (APR C-13: every effect on it ends, a Trainer's and an Ability's
/// included).
pub fn clear_declared(st: &mut State, p: usize, s: SlotId) {
    let on = LastingOn::pokemon(p, s);
    st.lasting.retain(|l| !(l.on == on && matches!(l.what, LastingWhat::Declared { .. } | LastingWhat::Recorded)));
}

/// `source` records the Pokémon in spot `s` of player `p` (once).
pub fn add_record(st: &mut State, p: usize, s: SlotId, source: CardId) {
    if !is_recorded(st, p, s, source) {
        st.lasting.push(LastingEffect { on: LastingOn::pokemon(p, s), source, what: LastingWhat::Recorded });
    }
}

/// Does `source` record the Pokémon in spot `s` of player `p`?
pub fn is_recorded(st: &State, p: usize, s: SlotId, source: CardId) -> bool {
    let on = LastingOn::pokemon(p, s);
    st.lasting.iter().any(|l| l.on == on && l.source == source && matches!(l.what, LastingWhat::Recorded))
}

/// `source` forgets the Pokémon in spot `s` of player `p`.
pub fn remove_record(st: &mut State, p: usize, s: SlotId, source: CardId) {
    let on = LastingOn::pokemon(p, s);
    st.lasting.retain(|l| !(l.on == on && l.source == source && matches!(l.what, LastingWhat::Recorded)));
}

/// `source` forgets every Pokémon of player `p`.
pub fn remove_records_of(st: &mut State, p: usize, source: CardId) {
    st.lasting.retain(|l| !(matches!(l.on, LastingOn::Pokemon(q, _) if q as usize == p) && l.source == source && matches!(l.what, LastingWhat::Recorded)));
}

/// The Pokémon in play that `source` records (the first, Active first; none when it left play).
pub fn recorded_slot(st: &State, source: CardId) -> Option<(usize, SlotId)> {
    for p in 0..2 {
        let pl = &st.players[p];
        for s in pl.in_play() {
            if !pl.slots[s as usize].cards().is_empty() && is_recorded(st, p, s, source) {
                return Some((p, s));
            }
        }
    }
    None
}

/// Put the declared effect `index` of card `source` on `on`, until `until` (one of a card and a declaration on the same
/// holder: it is the same effect).
pub fn add_declared(st: &mut State, on: LastingOn, source: CardId, index: u8, until: crate::state::Until) {
    if !st.lasting.iter().any(|l| l.on == on && l.source == source && matches!(l.what, LastingWhat::Declared { index: i, .. } if i == index)) {
        st.lasting.push(LastingEffect { on, source, what: LastingWhat::Declared { index, until } });
    }
}

/// Is there an effect cards declared and put for a time (`Op::Declare`)? One scan of the (short) lasting table, where the readers of
/// the declared effects would build their list.
pub fn has_declared(st: &State) -> bool {
    st.lasting.iter().any(|l| matches!(l.what, LastingWhat::Declared { .. }))
}

/// The declared effects that end with player `p`'s turn, before the end-of-turn effects they might be read by (APR E-21: the
/// end of a "this turn" effect is not an effect).
pub fn expire_this_turn(st: &mut State, p: usize) {
    st.lasting.retain(|l| !matches!(l.what, LastingWhat::Declared { until: crate::state::Until::ThisTurn(q), .. } if q as usize == p));
}

/// Every lasting effect on spot `s` of player `p` (the spot is emptied).
pub fn clear_spot(st: &mut State, p: usize, s: SlotId) {
    let on = LastingOn::pokemon(p, s);
    st.lasting.retain(|l| l.on != on);
}

/// The end of player `p`'s turn (`phase::end_turn`): the opponent's Pokémon lose what lasted "during your opponent's next
/// turn" from `p`'s last turn ("takes less damage", "no Weakness", Retaliate, the preventions); `p`'s Pokémon count a
/// turn of their locks, and what was put during this turn comes into force (the preventions, "no Weakness", Retaliate,
/// the Knock Out trigger, the next-turn bonus replacing the previous); the "attacks do less damage" on `p`'s Pokémon ends; the
/// TakesMore damage and the cost increases roll over (both players', `p`'s side first); `p`'s player locks count a turn.
pub fn end_turn(st: &mut State, p: usize) {
    // Every step below reads or rewrites an entry of the table: with none, there is nothing to end (a plain read).
    if st.lasting.is_empty() {
        return;
    }
    let o = 1 - p;
    // The "(This includes new Pokémon)" entries that last through player `p`'s turn end.
    st.lasting.retain(|l| !matches!(l.what, LastingWhat::Spawn { ends_with, .. } if ends_with as usize == p));
    for s in in_play(st, o) {
        let on = LastingOn::pokemon(o, s);
        st.lasting.retain(|l| !(l.on == on && matches!(l.what, LastingWhat::TakesLessDamage(_))));
        if let Some(LastingEffect { what: LastingWhat::NoWeakness { .. }, .. }) = find(st, on, Kind::NoWeakness) {
            modify(st, on, Kind::NoWeakness, |e| {
                if let LastingWhat::NoWeakness { armed, .. } = &mut e.what {
                    *armed = false;
                }
            });
        }
        set(st, on, Kind::RetaliateArmed, None);
        st.lasting.retain(|l| !(l.on == on && matches!(l.what, LastingWhat::Prevent { .. })));
    }
    for s in in_play(st, p) {
        let on = LastingOn::pokemon(p, s);
        // The locks on the Pokémon count this turn of its owner (events batch 7, D14).
        st.lasting.retain_mut(|l| {
            if l.on == on {
                if let LastingWhat::Lock { turns, .. } = &mut l.what {
                    if *turns != LastingWhat::UNTIL_REMOVED {
                        *turns -= 1;
                    }
                    return *turns > 0;
                }
            }
            true
        });
        // A prevention put during this turn is in force during the opponent's next turn.
        for l in st.lasting.as_mut_slice().iter_mut() {
            if l.on == on {
                if let LastingWhat::Prevent { pending, .. } = &mut l.what {
                    *pending = false;
                }
            }
        }
        if let Some(LastingEffect { what: LastingWhat::NoWeakness { pending: true, .. }, .. }) = find(st, on, Kind::NoWeakness) {
            modify(st, on, Kind::NoWeakness, |e| {
                e.what = LastingWhat::NoWeakness { armed: true, pending: false };
            });
        }
        if let Some(mut r) = find(st, on, Kind::RetaliatePending).copied() {
            set(st, on, Kind::RetaliatePending, None);
            if let LastingWhat::Retaliate { pending, .. } = &mut r.what {
                *pending = false;
            }
            set(st, on, Kind::RetaliateArmed, Some(r));
        }
        if let Some(LastingEffect { what: LastingWhat::DiscardAttackerEnergy { pending: true, .. }, .. }) = find(st, on, Kind::DiscardEnergy) {
            modify(st, on, Kind::DiscardEnergy, |e| {
                if let LastingWhat::DiscardAttackerEnergy { armed, pending, .. } = &mut e.what {
                    *armed = true;
                    *pending = false;
                }
            });
        }
        set(st, on, Kind::DealsLess, None);
        // The bonus armed during this turn replaces the previous one, or none is left.
        let pending = find(st, on, Kind::BonusPending).copied();
        set(st, on, Kind::BonusPending, None);
        set(
            st,
            on,
            Kind::BonusArmed,
            pending.map(|mut b| {
                if let LastingWhat::NextTurnBonus { pending, .. } = &mut b.what {
                    *pending = false;
                }
                b
            }),
        );
    }
    // "The Defending Pokémon takes N more damage": armed at the end of the defending player's turn, then cleared at the
    // end of the attacker's following turn.
    for q in [p, o] {
        for s in in_play(st, q) {
            let on = LastingOn::pokemon(q, s);
            if let Some(LastingEffect { what: LastingWhat::TakesMoreDamage { pending: true, attacker, .. }, .. }) = find(st, on, Kind::TakesMore) {
                if *attacker != Some(p as u8) {
                    modify(st, on, Kind::TakesMore, |e| {
                        if let LastingWhat::TakesMoreDamage { pending, .. } = &mut e.what {
                            *pending = false;
                        }
                    });
                }
            }
        }
    }
    for q in [p, o] {
        for s in in_play(st, q) {
            let on = LastingOn::pokemon(q, s);
            if let Some(LastingEffect { what: LastingWhat::TakesMoreDamage { amount, attacker, pending: false, .. }, .. }) = find(st, on, Kind::TakesMore) {
                if *attacker == Some(p as u8) && *amount > 0 {
                    modify(st, on, Kind::TakesMore, |e| {
                        if let LastingWhat::TakesMoreDamage { amount, attacker, .. } = &mut e.what {
                            *amount = 0;
                            *attacker = None;
                        }
                    });
                }
            }
        }
    }
    cost_increase_end_of_turn(st, p);
    // The locks on player `p` count this turn.
    let on = LastingOn::Player(p as u8);
    st.lasting.retain_mut(|l| {
        if l.on == on {
            if let LastingWhat::PlayerLock { turns, .. } = &mut l.what {
                *turns -= 1;
                return *turns > 0;
            }
        }
        true
    });
}

/// The cost increases: armed when the attacker ends its turn (in force during the defender's turn), gone when the
/// defender ends its turn. Phase 4b (R6).
fn cost_increase_end_of_turn(st: &mut State, p: usize) {
    let me = Some(p as u8);
    for pass in 0..2 {
        for q in [p, 1 - p] {
            for s in in_play(st, q) {
                if st.slot_pokemon(q, s).is_none() {
                    continue;
                }
                let on = LastingOn::pokemon(q, s);
                for k in [Kind::AttackCost, Kind::RetreatCost] {
                    let Some(e) = find(st, on, k).copied() else { continue };
                    let (LastingWhat::AttackCostIncrease { now, pending, attacker } | LastingWhat::RetreatCostIncrease { now, pending, attacker }) = e.what else { continue };
                    let (now, pending, attacker) = if pass == 0 {
                        if pending != 0 && attacker == me {
                            (pending, 0, attacker)
                        } else {
                            continue;
                        }
                    } else if attacker.is_some() && attacker != me && pending == 0 && now > 0 {
                        (0, pending, None)
                    } else {
                        continue;
                    };
                    let what = if k == Kind::AttackCost { LastingWhat::AttackCostIncrease { now, pending, attacker } } else { LastingWhat::RetreatCostIncrease { now, pending, attacker } };
                    set(st, on, k, Some(LastingEffect { what, ..e }));
                }
            }
        }
    }
}

/// After the end-of-turn state check of player `p`'s turn (`phase::after_end_turn`): the Knock Out trigger on the opponent's
/// Pokémon ends.
pub fn after_end_turn(st: &mut State, p: usize) {
    let o = 1 - p;
    for s in in_play(st, o) {
        if st.slot_pokemon(o, s).is_none() {
            continue;
        }
        set(st, LastingOn::pokemon(o, s), Kind::DiscardEnergy, None);
    }
}

#[cfg(test)]
mod declared_tests {
    //! The effects cards declare and put for a time (`LastingWhat::Declared`): who they are on, when they end.
    use super::*;
    use crate::state::{LastingOn, Until};

    fn put(st: &mut State, on: LastingOn, source: CardId, index: u8, until: Until) {
        add_declared(st, on, source, index, until);
    }

    /// "During this turn" ends with the turn of the player it is on, not with the other player's; the same card and declaration on the same holder is one effect.
    #[test]
    fn expiry() {
        let mut st = State::new();
        put(&mut st, LastingOn::Player(0), 1, 0, Until::ThisTurn(0));
        put(&mut st, LastingOn::Player(0), 1, 0, Until::ThisTurn(0));
        put(&mut st, LastingOn::Player(1), 2, 0, Until::ThisTurn(1));
        assert_eq!(st.lasting.len(), 2, "one effect per card, declaration and holder");
        expire_this_turn(&mut st, 0);
        assert_eq!(st.lasting.len(), 1, "player 0's turn ended");
        expire_this_turn(&mut st, 1);
        assert!(st.lasting.is_empty());
    }

    /// An effect on a Pokémon stays when the Pokémon moves to the Bench (an Ability's effect, id1651; a Trainer's, id2228) but
    /// ends when it evolves or is devolved (APR C-13) and with its spot.
    #[test]
    fn on_a_pokemon() {
        let mut st = State::new();
        put(&mut st, LastingOn::pokemon(0, 1), 1, 0, Until::ThisTurn(0));
        put(&mut st, LastingOn::pokemon(0, 2), 2, 0, Until::ThisTurn(0));
        remove_attack_effects(&mut st, 0, 1);
        clear_effects(&mut st, 0, 1);
        assert_eq!(st.lasting.len(), 2, "moving to the Bench keeps it");
        clear_declared(&mut st, 0, 1);
        assert_eq!(st.lasting.len(), 1, "evolving ends it");
        clear_spot(&mut st, 0, 2);
        assert!(st.lasting.is_empty(), "leaving play ends it");
    }
}
