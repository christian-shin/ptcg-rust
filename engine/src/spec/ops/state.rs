//! Lasting effects and markers: attack flags, "during your opponent's next
//! turn" effects (vocabulary v1 "Operations", state; "Core trackers").
//!
//! `Arm` writes the engine's own lasting-effect state (the slot fields that
//! roll over at the end of the turn, `effects.rs` effects); markers use the
//! engine's marker lists. No op here asks a question.

use super::super::run::{Flow, Frame};
use super::super::*;
use crate::effects::{AtkBase, Effect, SlotRef};
use crate::game::{Game, R};
use crate::list::{CardId, SVec};
use crate::markers::{MarkerName, SourceType, TargetScope};
use crate::prefabs::*;
use crate::prompts::Res;
use crate::types::{ct, Stage};

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
    fn filter(self) -> Option<crate::state::PreventFilter> {
        let mut f = crate::state::PreventFilter::default();
        match self {
            DamageSource::Any => return None,
            DamageSource::Stage(s) => f.source_stage = Some(s as u8),
            DamageSource::Evolution => f.source_stage = Some(crate::state::PreventFilter::SOURCE_IS_EVOLUTION),
            DamageSource::HasAbility => f.source_has_ability = true,
            DamageSource::BasicNonColorless => {
                f.source_stage = Some(Stage::Basic as u8);
                let mut types = SVec::new();
                for t in [ct::GRASS, ct::FIRE, ct::WATER, ct::LIGHTNING, ct::PSYCHIC, ct::FIGHTING, ct::DARK, ct::METAL, ct::FAIRY, ct::DRAGON] {
                    types.push(t);
                }
                f.source_card_types = Some(types);
            }
        }
        Some(f)
    }
}

/// What the opponent can't play.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Locked {
    Item,
    Supporter,
    Evolve,
    Stadium,
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
    /// The opponent can't play these cards from their hand during their next turn.
    OppCannotPlay(Locked),
    /// During the opponent's next turn, whenever they try to use a Trainer from
    /// their hand, they flip a coin; on tails it is discarded instead.
    CoinFlipCancelTrainer,
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

fn attack_base(g: &Game, atk: crate::effects::EffId, target: SlotRef) -> Option<AtkBase> {
    match *g.e(atk) {
        Effect::Attack { p, opp, attack, source, .. } => Some(AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target }),
        _ => None,
    }
}

/// `if (!player.active.cannotUseAttacksNextTurnPending.includes(name)) push(name)`.
pub fn push_cannot_use_attack(g: &mut Game, p: usize, name: &'static str) {
    let a = g.st.players[p].active;
    let v = &mut g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending;
    if !v.contains(&name) {
        v.push(name);
    }
}

/// `DEFENDING_POKEMON_DOES_LESS_DAMAGE(store, state, effect, source, reduction)`.
pub fn defending_pokemon_does_less_damage(g: &mut Game, atk: crate::effects::EffId, reduction: i32) -> R {
    let b = match *g.e(atk) {
        Effect::Attack { opp, .. } => {
            let target = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
            attack_base(g, atk, target)
        }
        _ => None,
    };
    if let Some(b) = b {
        g.run_fx(Effect::ReduceDamage { b, reduction })?;
    }
    Ok(())
}

fn this_attack_name(g: &Game, atk: crate::effects::EffId) -> &'static str {
    match *g.e(atk) {
        Effect::Attack { attack, .. } => crate::engine::attack::attack_def(g, attack).name,
        _ => "",
    }
}

fn arm(g: &mut Game, me: CardId, f: &Frame, what: Lasting) -> R {
    let atk = f.eff;
    let (p, source) = match *g.e(atk) {
        Effect::Attack { p, source, .. } => (p as usize, source),
        _ => return Ok(()),
    };
    match what {
        Lasting::PreventRetreat => block_retreat(g, atk)?,
        Lasting::CannotAttackNextTurn => {
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
        Lasting::CannotUseThisAttackNextTurn => {
            let name = this_attack_name(g, atk);
            push_cannot_use_attack(g, p, name);
        }
        Lasting::BlockThisAttackUntilLeavesActive => {
            let name = this_attack_name(g, atk);
            if let Some(b) = attack_base(g, atk, source) {
                g.run_fx(Effect::PreventAttackUntilLeavesActive { b, name })?;
            }
        }
        Lasting::TakesLessDamage(n) => {
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].damage_reduction_next_turn = n;
        }
        Lasting::DealsLessDamage(n) => defending_pokemon_does_less_damage(g, atk, n)?,
        Lasting::TakesMoreDamage(n) => {
            let o = 1 - p;
            let target = SlotRef::new(o, g.st.players[o].active);
            if let Some(b) = attack_base(g, atk, target) {
                g.run_fx(Effect::DefendingPokemonTakesMoreDamage { b, damage_bonus: n })?;
            }
        }
        Lasting::PreventDamage(src) => match src.filter() {
            Some(filter) => prevent_damage_filtered(g, atk, filter)?,
            None => prevent_damage(g, atk)?,
        },
        Lasting::Retaliate(n) => {
            if let Some(b) = attack_base(g, atk, source) {
                g.run_fx(Effect::RetaliateOnDamage { b, damage: n, source_card: me })?;
            }
        }
        Lasting::OppCannotPlay(l) => {
            let locks = match l {
                Locked::Item => crate::effects::play_lock::ITEM,
                Locked::Supporter => crate::effects::play_lock::SUPPORTER,
                Locked::Evolve => crate::effects::play_lock::EVOLVE,
                Locked::Stadium => crate::effects::play_lock::STADIUM,
            };
            opponent_cannot_play_cards(g, atk, locks)?;
        }
        Lasting::DiscardAttackerEnergyIfKnockedOut => discard_attacker_energy_if_knocked_out(g, atk, me)?,
        Lasting::CoinFlipCancelTrainer => {
            if let Some(b) = attack_base(g, atk, source) {
                g.run_fx(Effect::CoinFlipCancelTrainerPlay { b })?;
            }
        }
        Lasting::IncreaseAttackCost | Lasting::IncreaseRetreatCost => {
            // The pending value is written first, so it survives a prevented effect (Twinleaf).
            let o = 1 - p;
            let a = g.st.players[o].active;
            let slot = &mut g.st.players[o].slots[a as usize];
            if what == Lasting::IncreaseAttackCost {
                slot.attack_cost_increase_next_turn_pending = 1;
            } else {
                slot.retreat_cost_increase_next_turn_pending = 1;
            }
            if let Some(b) = attack_base(g, atk, SlotRef::new(o, a)) {
                if what == Lasting::IncreaseAttackCost {
                    g.run_fx(Effect::IncreaseAttackCostNextTurn { b })?;
                } else {
                    g.run_fx(Effect::IncreaseRetreatCostNextTurn { b })?;
                }
            }
        }
        Lasting::NoWeakness => {
            if let Some(b) = attack_base(g, atk, source) {
                g.run_fx(Effect::ThisPokemonHasNoWeakness { b })?;
            }
        }
        Lasting::PreventAttackEffects => prevent_effects_of_attacks(g, atk)?,
        Lasting::SelfCannotRetreat => block_self_retreat(g, atk)?,
        Lasting::OppSmallEnergyCannotAttack(n) => opponent_pokemon_with_x_or_less_energy_cannot_attack(g, atk, n)?,
    }
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
        Op::SetFlag(s) => {
            let p = f.who(s.who);
            match s.flag {
                PlayerFlag::AncientSupporter => g.st.players[p].ancient_supporter = s.value,
            }
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerFlag {
    /// "The player played an Ancient Supporter this turn."
    AncientSupporter,
}

/// Set or clear a flag of a player.
pub struct SetFlagSpec {
    pub who: Who,
    pub flag: PlayerFlag,
    pub value: bool,
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
