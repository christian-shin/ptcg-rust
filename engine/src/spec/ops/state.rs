//! Lasting effects and markers: attack flags, "during your opponent's next
//! turn" effects (vocabulary v1 "Operations", state; "Core trackers").
//!
//! `Arm` writes the engine's own lasting-effect state (the slot fields that
//! roll over at the end of the turn, `effects.rs` effects); markers use the
//! engine's marker lists. No op here asks a question.

use super::super::run::{Flow, Frame};
use super::super::*;
use crate::effects::Effect;
use crate::game::{Game, R};
use crate::list::CardId;
use crate::markers::{MarkerName, SourceType, TargetScope};
use crate::prefabs::*;
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
    Barrage,
    FirstTurnAllowed,
    /// Festival Lead: this attack's Barrage flag is "Festival Grounds is in play", and is switched
    /// off while the Ability is blocked. `value`: the printed attack has a `barrage: false` key
    /// (Dipplin), so the flag only shows in the card while it is on; otherwise any write shows.
    FestivalLead,
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
    /// The Defending Pokémon can't retreat during the opponent's next turn (an
    /// attack effect on the opponent's Pokémon).
    PreventRetreat,
    /// This Pokémon can't attack during your next turn.
    CannotAttackNextTurn,
    /// This Pokémon can't use this attack during your next turn.
    CannotUseThisAttackNextTurn,
    /// This Pokémon can't use this attack again until it leaves the Active Spot.
    BlockThisAttackUntilLeavesActive,
    /// The Defending Pokémon can't use the attack of this name during the opponent's next turn (APR C-15: "it can't be
    /// used during their next turn" after the attacker chose it).
    CannotUseAttack(&'static str),
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
    /// The opponent can't do these actions with these cards (a [`LockDecl`], the declaration an in-play lock
    /// uses) during their next turn.
    OppCannotPlay(&'static LockDecl),
    // --- S3 agent 3 appends ---
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
    /// This Pokémon can't retreat during your next turn.
    SelfCannotRetreat,
    /// During the opponent's next turn, Pokémon with this many Energy or fewer can't attack.
    OppSmallEnergyCannotAttack(i32),
}

/// Arm a lasting effect of the attack being used.
pub struct ArmSpec {
    pub what: Lasting,
}

// ---------------------------------------------------------------------------
// Markers

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MarkerScope {
    /// A marker on a player (`Who::Me` = the player the program runs for).
    Player(Who),
    /// A marker on a Pokémon (a Trainer effect when the source is `TrainerEffect`: it stays on
    /// the Pokémon when it moves or evolves).
    Slot(SlotExpr),
    /// A marker on every Pokémon of a player.
    EveryPokemon(Who),
}

/// Which markers of a name count: only the one set by this card, or any.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MarkerFrom {
    This,
    Any,
}

pub struct SetMarkerSpec {
    pub scope: MarkerScope,
    pub name: &'static str,
    pub source: RuleSource,
}

pub struct ClearMarkerSpec {
    pub scope: MarkerScope,
    pub name: &'static str,
    pub from: MarkerFrom,
}

pub(crate) fn marker_of(name: &'static str) -> MarkerName {
    crate::markers::intern(name)
}

/// Does the player's marker list hold the marker `name` (set by `me` only for `From::This`)?
pub fn has_marker(g: &Game, me: CardId, p: usize, name: &'static str, from: MarkerFrom) -> bool {
    let Some(id) = crate::markers::marker_id(name) else { return false };
    match from {
        MarkerFrom::This => g.st.players[p].marker.has_from(id, me),
        MarkerFrom::Any => g.st.players[p].marker.has(id),
    }
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
    crate::engine::apply::apply_effect(g, p, target, what, me, attack, f.cause)?;
    Ok(())
}

pub(crate) fn exec(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
        Op::AttackFlag(a) if a.flag == AttackFlagKind::Barrage => {
            // Festival Lead: the attack may be used twice (read by the core's attack use).
            crate::copy_attack::write_barrage(g, me, |b, shown| {
                if a.value {
                    *b |= 1;
                } else {
                    *b &= !1;
                }
                *shown |= 1;
            });
            Ok(Flow::Next)
        }
        Op::AttackFlag(a) => {
            if let Effect::Attack { ignore_weakness, ignore_resistance, ignore_defender_effects, .. } = g.e_mut(f.eff) {
                match a.flag {
                    AttackFlagKind::NoWeakness => *ignore_weakness = a.value,
                    AttackFlagKind::NoResistance => *ignore_resistance = a.value,
                    AttackFlagKind::IgnoreDefenderEffects => *ignore_defender_effects = a.value,
                    AttackFlagKind::FestivalLead => {}
                    _ => unimplemented!("spec attack flag not implemented yet (ops/state.rs)"),
                }
            }
            if a.flag == AttackFlagKind::FestivalLead {
                festival_lead_flag(g, f, me, a.value);
            }
            Ok(Flow::Next)
        }
        Op::Arm(a) => {
            arm(g, me, f, a.what)?;
            Ok(Flow::Next)
        }
        Op::AbilityUsed(a) => {
            if let Some(marker) = a.marker {
                // A once-per-turn Ability that counts as used from here (after its cost or choice).
                use_ability_once_per_turn(g, f.p as usize, marker_of(marker), me)?;
            }
            ability_used(g, f.p as usize, me);
            Ok(Flow::Next)
        }
        Op::SetMarker(m) => {
            let w = match m.scope {
                MarkerScope::Player(w) => w,
                MarkerScope::Slot(e) => {
                    if let Some(t) = slot_of(g, me, f, e) {
                        let source = if m.source == RuleSource::TrainerEffect { SourceType::Trainer } else { SourceType::None };
                        g.st.players[t.p as usize].slots[t.s as usize].marker.add(marker_of(m.name), me, source, TargetScope::Pokemon);
                    }
                    return Ok(Flow::Next);
                }
                MarkerScope::EveryPokemon(w) => {
                    let p = f.who(w);
                    for s in g.st.players[p].in_play().iter().copied().collect::<Vec<_>>() {
                        g.st.players[p].slots[s as usize].marker.add(marker_of(m.name), me, SourceType::Trainer, TargetScope::Pokemon);
                    }
                    return Ok(Flow::Next);
                }
            };
            let p = f.who(w);
            let source = match m.source {
                RuleSource::Ability => SourceType::Ability,
                RuleSource::Energy => SourceType::Energy,
                RuleSource::Stadium => SourceType::Stadium,
                RuleSource::TrainerEffect => SourceType::Trainer,
                _ => SourceType::None,
            };
            let _ = source;
            g.st.players[p].marker.add(marker_of(m.name), me, SourceType::None, TargetScope::None);
            Ok(Flow::Next)
        }
        Op::ClearMarker(m) => {
            let w = match m.scope {
                MarkerScope::Player(w) => w,
                MarkerScope::EveryPokemon(w) => {
                    let p = f.who(w);
                    let id = marker_of(m.name);
                    for s in g.st.players[p].in_play().iter().copied().collect::<Vec<_>>() {
                        let mk = &mut g.st.players[p].slots[s as usize].marker;
                        match m.from {
                            MarkerFrom::This => mk.remove_from(id, me),
                            MarkerFrom::Any => mk.remove(id),
                        }
                    }
                    return Ok(Flow::Next);
                }
                MarkerScope::Slot(e) => {
                    if let Some(t) = slot_of(g, me, f, e) {
                        let mk = &mut g.st.players[t.p as usize].slots[t.s as usize].marker;
                        match m.from {
                            MarkerFrom::This => mk.remove_from(marker_of(m.name), me),
                            MarkerFrom::Any => mk.remove(marker_of(m.name)),
                        }
                    }
                    return Ok(Flow::Next);
                }
            };
            let p = f.who(w);
            let id = marker_of(m.name);
            match m.from {
                MarkerFrom::This => {
                    if g.st.players[p].marker.has_from(id, me) {
                        g.st.players[p].marker.remove_from(id, me);
                    }
                }
                MarkerFrom::Any => g.st.players[p].marker.remove(id),
            }
            Ok(Flow::Next)
        }
        _ => unreachable!("not a state op"),
    }
}

pub(crate) fn resume(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op, _results: &[Res]) -> R<Flow> {
    Ok(Flow::Next)
}

/// The Ability of this Pokémon counts as used (the board effect shown on it), when the use
/// succeeded rather than when it started. With `marker`, the player's marker named so is set by this
/// card too (the Ability is refused when it already is); clear it with an end-of-turn trigger.
pub struct AbilityUsedSpec {
    pub marker: Option<&'static str>,
}


/// The Barrage flag of the attack being used (Festival Lead): on while Festival Grounds is in play
/// and the Ability isn't blocked.
fn festival_lead_flag(g: &mut Game, f: &Frame, me: CardId, pristine_has_key: bool) {
    let idx = match *g.e(f.eff) {
        Effect::Attack { attack, .. } => attack.idx(),
        _ => return,
    };
    let on = !crate::prefabs::is_ability_blocked(g, f.p as usize, me, None) && g.st.stadium_card().map(|s| g.st.cdef(s).name == "Festival Grounds").unwrap_or(false);
    crate::copy_attack::write_barrage(g, me, |b, shown| {
        if on {
            *b |= 1 << idx;
        } else {
            *b &= !(1 << idx);
        }
        if on || !pristine_has_key {
            *shown |= 1 << idx;
        } else {
            *shown &= !(1 << idx);
        }
    });
}
