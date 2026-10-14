//! Lasting effects, declared effects, usage and slot records: attack flags, "during your opponent's next
//! turn" effects (vocabulary v1 "Operations", state; "Core trackers").
//!
//! `Arm` writes the engine's own lasting-effect state (the lasting table, `engine::lasting`); `Declare` puts an effect the
//! card declares there; `Used` makes a usage record (`engine::usage`); `Record` / `Forget` make and end a card's record of a
//! Pokémon. No op here asks a question.

use super::super::run::{Flow, Frame};
use super::super::*;
use crate::effects::{ApplyTarget, Effect, SlotRef};
use crate::game::{Game, R};
use crate::list::{CardId, SVec};
use crate::prompts::Res;
use crate::types::Stage;

// ---------------------------------------------------------------------------
// AttackFlag

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttackFlagKind {
    NoWeakness,
    NoResistance,
    /// Shred: effects on the Defending Pokémon don't change this attack's damage.
    IgnoreDefenderEffects,
    PreventDamage,
    FirstTurnAllowed,
    /// The player took the attack's optional effect ("you may ...": Dialga's Chrono Burst): read by the attack's text after
    /// the damage (`Cond::AttackFlag`).
    OptionChosen,
}

/// Write a flag of the attack being used (text that runs before the damage).
pub struct AttackFlagSpec {
    pub flag: AttackFlagKind,
    pub value: bool,
}

// ---------------------------------------------------------------------------
// Arm

/// Where the attack's damage comes from, for "prevent all damage done by
/// attacks from ...".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DamageSource {
    Any,
    Stage(Stage),
    Evolution,
    HasAbility,
    // --- S3-4 appends ---
    /// Basic Pokémon that are not [C] (Crown Opal).
    BasicNonColorless,
}

impl DamageSource {
    /// The lasting `Prevent` "during your opponent's next turn, prevent all damage done to this Pokémon by attacks from
    /// <these> Pokémon" (the attacking Pokémon where it is now: `CausePred::Pokemon`).
    pub const fn spec(self) -> &'static crate::spec::passive::PreventSpec {
        use crate::spec::passive as ps;
        match self {
            DamageSource::Any => &ps::LASTING_PREVENT_DAMAGE,
            DamageSource::Stage(Stage::Basic) => &ps::LASTING_PREVENT_DAMAGE_FROM_BASIC,
            DamageSource::Stage(_) | DamageSource::Evolution => &ps::LASTING_PREVENT_DAMAGE_FROM_EVOLUTION,
            DamageSource::HasAbility => &ps::LASTING_PREVENT_DAMAGE_FROM_ABILITY,
            DamageSource::BasicNonColorless => &ps::LASTING_PREVENT_DAMAGE_FROM_BASIC_NON_COLORLESS,
        }
    }
}

/// A lasting attack effect (vocabulary v1 `Lasting`). The core owns the
/// pending-to-active roll-over at the end of the turn and the expiry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lasting {
    /// A lock the attack leaves on a Pokémon or on the opponent, for a time (events batch 7, user decision D14): "this
    /// Pokémon can't attack during your next turn", "can't use [Attack Name]" (APR C-15), "the Defending Pokémon can't
    /// retreat", "your opponent can't play Item cards from their hand" (APR C-19), Seismitoad's coin-gated one.
    Lock(LastingLockSpec),
    /// During the opponent's next turn, this Pokémon takes n less damage from attacks.
    TakesLessDamage(i32),
    /// During the opponent's next turn, attacks used by the Defending Pokémon do n less damage.
    DealsLessDamage(i32),
    /// During the opponent's next turn, the Defending Pokémon takes n more damage from attacks.
    TakesMoreDamage(i32),
    /// During the opponent's next turn, prevent all damage done to this Pokémon by attacks from `source`.
    PreventDamage(DamageSource),
    /// During the opponent's next turn, if this Pokémon is damaged by an attack
    /// (even if Knocked Out), put n damage counters on the Attacking Pokémon.
    Retaliate(i32),
    /// During the opponent's next turn, attacks used by the Defending Pokémon cost [C] more.
    IncreaseAttackCost,
    /// During the opponent's next turn, the Defending Pokémon's Retreat Cost is [C] more.
    IncreaseRetreatCost,
    /// During the opponent's next turn this Pokémon has no Weakness.
    NoWeakness,
    /// During the opponent's next turn, prevent all effects of attacks done to this Pokémon.
    PreventAttackEffects,
    /// During the opponent's next turn, if this Pokémon is Knocked Out by damage from an attack,
    /// the attacker's controller discards an Energy attached to the Attacking Pokémon (Little Grudge).
    DiscardAttackerEnergyIfKnockedOut,
}

/// A lock an attack leaves (`Lasting::Lock`): the declaration (what it forbids, a [`LockDecl`] as an in-play lock
/// uses), whom it is on, until when, and the attack it names.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LastingLockSpec {
    pub lock: &'static LockDecl,
    pub on: LockOn,
    pub until: LockUntil,
    pub attack: NamedAttack,
}

/// Whom a lasting lock is on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LockOn {
    /// The attacking Pokémon ("this Pokémon").
    ThisPokemon,
    /// The opponent's Active Pokémon ("the Defending Pokémon").
    DefendingPokemon,
    /// The opponent ("your opponent can't ..."; a player has no spot: no `Prevent` matches it, APR C-19).
    Opponent,
}

/// Until when a lasting lock lasts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LockUntil {
    /// "During your next turn" (a lock on the attacking Pokémon: the attack ends this turn).
    YourNextTurn,
    /// "During your opponent's next turn".
    OpponentsNextTurn,
    /// "Until it leaves the Active Spot" (ends with the Pokémon's other effects of attacks, APR C-15).
    LeavesActive,
}

/// The attack a "can't use [Attack Name]" lock names (APR C-15: the Pokémon's other attacks can still be used).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NamedAttack {
    /// None: the declaration says what it forbids.
    Any,
    /// The attack being used ("this Pokémon can't use [this attack] during your next turn").
    This,
    /// An attack the attacker chose ("choose 1 of the Defending Pokémon's attacks; it can't be used ..."): its name is
    /// the one forbidden.
    Chosen(crate::state::AttackRef),
}

impl LastingLockSpec {
    /// "Your opponent can't ... during their next turn" (APR C-19).
    pub const fn on_opponent(lock: &'static LockDecl) -> LastingLockSpec {
        LastingLockSpec { lock, on: LockOn::Opponent, until: LockUntil::OpponentsNextTurn, attack: NamedAttack::Any }
    }
    /// "This Pokémon can't ..." until `until`.
    pub const fn on_this_pokemon(lock: &'static LockDecl, until: LockUntil) -> LastingLockSpec {
        LastingLockSpec { lock, on: LockOn::ThisPokemon, until, attack: NamedAttack::Any }
    }
    /// "The Defending Pokémon can't ... during your opponent's next turn".
    pub const fn on_defending(lock: &'static LockDecl) -> LastingLockSpec {
        LastingLockSpec { lock, on: LockOn::DefendingPokemon, until: LockUntil::OpponentsNextTurn, attack: NamedAttack::Any }
    }
    /// The lock names an attack ("can't use [Attack Name]").
    pub const fn naming(self, attack: NamedAttack) -> LastingLockSpec {
        LastingLockSpec { attack, ..self }
    }
}

/// Arm a lasting effect of the attack being used.
pub struct ArmSpec {
    pub what: Lasting,
}

/// "During your opponent's next turn, [each of a player's Pokémon matching `subject`] ... (This includes new Pokémon that
/// come into play.)" (Walrein's Frigid Fangs, Iron Defender; user decision 2026-10-12, batch 8 C12c): the effect `what`
/// is put on each of those Pokémon in play when it resolves, one ApplyEffect event each (a Pokémon that prevents the
/// effects of attacks keeps it off: JP FAQ Walrein / Mist Energy), and on each new Pokémon that comes into play while
/// it lasts (the parenthetical; `engine::lasting::spawn_on_entering`). Each one is an effect on that Pokémon (APR B-05:
/// Iron Defender is "an effect on the Pokémon taking the damage"): an attack's ends when the Pokémon moves to the Bench,
/// evolves or devolves (APR C-15); it lasts through the opponent's next turn.
pub struct ArmEachSpec {
    /// Whose Pokémon: the user's (`Who::Me`) or the opponent's.
    pub side: Who,
    pub subject: SlotPred,
    pub what: Lasting,
}

// ---------------------------------------------------------------------------
// Declared effects

/// Whom a declared effect is on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeclareOn {
    /// A player ("during this turn, attacks used by your Pokémon ...": a Trainer's effect is on its player, APR C-19).
    Player(Who),
    /// A Pokémon ("this turn, attacks used by this Pokémon ...": an Ability's effect on its own Pokémon, APR C-15; "choose 1 of
    /// your Pokémon": a Trainer's effect on a Pokémon, Acerola's Mischief): it stays when the Pokémon moves to the Bench and
    /// ends with the Pokémon's effects (evolving, devolving, leaving play).
    Pokemon(SlotExpr),
}

/// Until when a declared effect lasts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeclareUntil {
    /// "During this turn": it ends when the turn ends, before the end-of-turn effects (APR E-21).
    EndOfTurn,
    /// Until the player's next Pokémon Checkup (Briar: "this turn, if ... Knocked Out ..." is read by the state check after the
    /// attack's turn ends).
    Checkup,
    /// "During your opponent's next turn": it ends when the user's opponent's turn ends.
    OpponentsNextTurn,
}

/// The card forgets what it recorded (`Op::Forget`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ForgetSpec {
    /// The Pokémon (the card's record of it).
    Pokemon(SlotExpr),
    /// Every Pokémon of the player.
    Every(Who),
}

/// `effect` of `CardSpec::effects` is put on `on` until `until`.
pub struct DeclareSpec {
    pub effect: u8,
    pub on: DeclareOn,
    pub until: DeclareUntil,
}

// ---------------------------------------------------------------------------
// Usage records

/// What a usage record of the card's program is about.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UsedWhat {
    /// The printed Ability (or Pokémon Power) number `n` of this card: the record is keyed by its printed name, so "you can't
    /// use more than 1 X Ability each turn" is the same record for every copy (`UsedBy::Player`).
    Ability(u8),
    /// This card's own record ("attached this turn": Ignition Energy).
    Itself,
}

/// Whose record it is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UsedBy {
    /// This card's: "once during your turn" for each copy (a Pokémon's own use ends with the Pokémon: id317).
    Copy,
    /// The program's player's: the Ability of every copy ("you can't use more than 1 X Ability each turn").
    Player,
    /// The Pokémon the program is about (the one picked, the Knocked Out one): "the effect doesn't stack" (id2083), for as
    /// long as it is in play.
    Picked,
}

/// When the record ends.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UsedUntil {
    /// This turn, ending before the end-of-turn effects (APR E-21).
    ThisTurn,
    /// This turn, once the end-of-turn effects that read it are done.
    AfterTurn,
    /// Until the card or Pokémon it is about leaves play.
    Removed,
}

/// A usage record, read by `Cond::Used` and written by `Op::Used` ("this Ability counts as used from here": after its cost
/// or choice, when the card says it counts only when something happened).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UsedSpec {
    pub what: UsedWhat,
    pub by: UsedBy,
    pub until: UsedUntil,
}

impl UsedSpec {
    /// The use of the card's printed Ability number `n`, by `by`, for this turn.
    pub const fn ability(n: u8, by: UsedBy) -> UsedSpec {
        UsedSpec { what: UsedWhat::Ability(n), by, until: UsedUntil::ThisTurn }
    }
}

/// The record `u` stands for in the program of `me` (`None`: the Pokémon it is about isn't there).
pub(crate) fn used_record(g: &Game, me: CardId, f: &Frame, u: &UsedSpec) -> Option<(crate::state::UsageKey, crate::state::UsageBy, crate::state::Until)> {
    use crate::state::{Until, UsageBy};
    let key = match u.what {
        UsedWhat::Ability(n) => crate::engine::usage::ability_key(&g.st, me, n),
        UsedWhat::Itself => crate::engine::usage::declared_key(&g.st, me, 0xFF),
    };
    let by = match u.by {
        UsedBy::Copy => UsageBy::Card(me),
        UsedBy::Player => UsageBy::Player(f.p),
        UsedBy::Picked => {
            let s = crate::spec::value::slot_of(g, me, f, SlotExpr::Picked)?;
            UsageBy::Card(g.st.slot_pokemon(s.p as usize, s.s)?)
        }
    };
    let until = match u.until {
        UsedUntil::ThisTurn => Until::ThisTurn(f.p),
        UsedUntil::AfterTurn => Until::AfterTurn(f.p),
        UsedUntil::Removed => Until::Removed,
    };
    Some((key, by, until))
}

/// Is there a record `u` stands for?
pub fn is_used(g: &Game, me: CardId, f: &Frame, u: &UsedSpec) -> bool {
    used_record(g, me, f, u).map_or(false, |(key, by, _)| crate::engine::usage::has(&g.st, key, by))
}

// ---------------------------------------------------------------------------
// Execution

/// Arm a lasting effect of the attack being used: the ApplyEffect event (`engine::apply`) on its target, by the frame's
/// cause.
fn arm(g: &mut Game, me: CardId, f: &Frame, what: Lasting) -> R {
    let (p, source, attack) = match *g.e(f.eff) {
        Effect::Attack { p, source, attack, .. } => (p as usize, source, attack),
        _ => return Ok(()),
    };
    let target = crate::engine::apply::target_of(g, p, source, what);
    crate::engine::apply::apply_effect(g, p, target, what, me, Some(attack), f.cause)?;
    Ok(())
}

impl std::fmt::Debug for ArmEachSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArmEach({:?}, {:?})", self.side, self.what)
    }
}

/// `Op::ArmEach`: the effect on each matching Pokémon of the side now, and the entry that puts it on the new ones.
fn arm_each(g: &mut Game, me: CardId, f: &Frame, a: &'static ArmEachSpec) -> R {
    let (p, attack) = match *g.e(f.eff) {
        Effect::Attack { p, attack, .. } => (p as usize, Some(attack)),
        _ => (f.p as usize, None),
    };
    let q = f.who(a.side);
    let slots: SVec<crate::state::SlotId, { crate::state::MAX_SLOTS }> = g.st.players[q].in_play();
    for s in slots.iter().copied() {
        let t = SlotRef::new(q, s);
        if g.st.slot_pokemon(q, s).is_none() || !crate::spec::value::slot_pred_m(g, me, t, &a.subject)? {
            continue;
        }
        crate::engine::apply::apply_effect(g, p, ApplyTarget::Slot(t), a.what, me, attack, f.cause)?;
    }
    // It lasts through the opponent's next turn: it ends at the end of the turn of the user's opponent.
    crate::engine::lasting::add_spawn(&mut g.st, q, a, me, f.cause, attack, (1 - p) as u8);
    g.st.bump();
    Ok(())
}

pub(crate) fn exec(g: &mut Game, me: CardId, f: &mut Frame, op: &'static Op) -> R<Flow> {
    match op {
        Op::AttackFlag(a) if a.flag == AttackFlagKind::OptionChosen => {
            if (f.eff as usize) < g.fx.len() {
                if a.value {
                    g.set_fx_flag(f.eff, crate::game::fx_flag::OPTION_CHOSEN);
                } else {
                    g.fx.as_mut_slice()[f.eff as usize].flags &= !crate::game::fx_flag::OPTION_CHOSEN;
                }
            }
            Ok(Flow::Next)
        }
        Op::AttackFlag(a) => {
            if let Effect::Attack { ignore_weakness, ignore_resistance, ignore_defender_effects, .. } = g.e_mut(f.eff) {
                match a.flag {
                    AttackFlagKind::NoWeakness => *ignore_weakness = a.value,
                    AttackFlagKind::NoResistance => *ignore_resistance = a.value,
                    AttackFlagKind::IgnoreDefenderEffects => *ignore_defender_effects = a.value,
                    _ => unimplemented!("spec attack flag not implemented yet (ops/state.rs)"),
                }
            }
            Ok(Flow::Next)
        }
        Op::Arm(a) => {
            arm(g, me, f, a.what)?;
            Ok(Flow::Next)
        }
        Op::ArmEach(a) => {
            arm_each(g, me, f, a)?;
            Ok(Flow::Next)
        }
        Op::Declare(d) => {
            let on = match d.on {
                DeclareOn::Player(w) => crate::state::LastingOn::Player(f.who(w) as u8),
                DeclareOn::Pokemon(e) => match slot_of(g, me, f, e) {
                    Some(s) => crate::state::LastingOn::pokemon(s.p as usize, s.s),
                    None => return Ok(Flow::Next),
                },
            };
            // The user's turn: the program's player.
            let until = match d.until {
                DeclareUntil::EndOfTurn => crate::state::Until::ThisTurn(f.p),
                DeclareUntil::Checkup => crate::state::Until::Checkup(f.p),
                DeclareUntil::OpponentsNextTurn => crate::state::Until::ThisTurn(1 - f.p),
            };
            crate::engine::lasting::add_declared(&mut g.st, on, me, d.effect, until);
            Ok(Flow::Next)
        }
        Op::Record(e) => {
            if let Some(s) = slot_of(g, me, f, *e) {
                crate::engine::lasting::add_record(&mut g.st, s.p as usize, s.s, me);
            }
            Ok(Flow::Next)
        }
        Op::Forget(x) => {
            match *x {
                ForgetSpec::Pokemon(e) => {
                    if let Some(s) = slot_of(g, me, f, e) {
                        crate::engine::lasting::remove_record(&mut g.st, s.p as usize, s.s, me);
                    }
                }
                ForgetSpec::Every(w) => crate::engine::lasting::remove_records_of(&mut g.st, f.who(w), me),
            }
            Ok(Flow::Next)
        }
        Op::Used(u) => {
            if let Some((key, by, until)) = used_record(g, me, f, u) {
                crate::engine::usage::record(&mut g.st, key, by, until);
            }
            Ok(Flow::Next)
        }
        _ => unreachable!("not a state op"),
    }
}

pub(crate) fn resume(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op, _results: &[Res]) -> R<Flow> {
    Ok(Flow::Next)
}
