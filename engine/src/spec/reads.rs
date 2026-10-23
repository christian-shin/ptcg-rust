//! What a declaration reads (speed batch L3): the classes of state (`state::reads`) and the derived facts
//! (`derived::fact`) the reading of a condition, number, predicate or fact modifier can depend on. The derived layer keys
//! each fact's memo by the counters of the classes its game's declarations read (`derived.rs`, "Invalidation"); a reading
//! this walk can't place in a class is `reads::ANY` (the fact is then keyed by every write, `State::version`).
//!
//! Every match below is exhaustive (no `_`): a new variant of the vocabulary doesn't compile until it is classified.
//! A program's own state (card registers, the picked Pokémon, the last op's outcome, coins, the effect in progress) and
//! the "this turn" / "last turn" records are no class: `ANY`.

use super::passive::{ActiveLockSpec, AbilityLockSpec, CostChange, LockWhile, LockerProbe, Locker, Modifier, Passive, PreventWhat, RuleSource};
use super::value::{Cond, EnergyUnit, Num, Pred, SlotExpr, SlotPred, SlotSel, Zone, ZoneRef};
use crate::derived::fact;
use crate::state::reads::{ANY, BOARD, CONDITIONS, COUNTERS, EFFECTS, TURN, ZONES};

/// The state classes (`state::reads`) and the derived facts (a bit per `derived::fact`) a reading depends on.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct ReadSet {
    pub state: u8,
    pub facts: u16,
}

impl ReadSet {
    pub const NONE: ReadSet = ReadSet { state: 0, facts: 0 };

    const fn st(state: u8) -> ReadSet {
        ReadSet { state, facts: 0 }
    }

    const fn fact(f: usize) -> ReadSet {
        ReadSet { state: 0, facts: 1 << f }
    }

    #[inline]
    pub fn or(self, o: ReadSet) -> ReadSet {
        ReadSet { state: self.state | o.state, facts: self.facts | o.facts }
    }
}

fn all<T>(xs: &[T], f: impl Fn(&T) -> ReadSet) -> ReadSet {
    xs.iter().fold(ReadSet::NONE, |a, x| a.or(f(x)))
}

/// A card predicate: the card's definition (fixed for the game) and, for the variants that ask "is it a Pokémon" (a Fossil
/// in play is one) or look at the Pokémon in play, the board.
pub fn pred(p: &Pred) -> ReadSet {
    match p {
        Pred::Not(q) => pred(q),
        Pred::All(qs) | Pred::OneOf(qs) => all(qs, pred),
        Pred::Pokemon
        | Pred::Basic
        | Pred::HpAtMost(_)
        | Pred::HasAttacks
        | Pred::Stage(_)
        | Pred::HasAbilityNamed(_)
        | Pred::StageIs(_)
        | Pred::PokemonType(_)
        | Pred::HasResistance(_)
        | Pred::EvolvesFrom(_)
        | Pred::EvolvesFromOwnInPlay
        | Pred::EvolvesFromActive => ReadSet::st(BOARD),
        Pred::Any
        | Pred::False
        | Pred::Energy
        | Pred::BasicEnergy
        | Pred::Trainer
        | Pred::Item
        | Pred::Supporter
        | Pred::Tool
        | Pred::Stadium
        | Pred::Name(_)
        | Pred::Tag(_)
        | Pred::NameContains(_)
        | Pred::HasAttackNamed(_)
        | Pred::ProvidesType(_)
        | Pred::Provides(_)
        | Pred::RuleBox
        | Pred::PokemonEx
        | Pred::PokemonV
        | Pred::SpecialEnergy
        | Pred::PrintsAbility => ReadSet::NONE,
    }
}

/// A Pokémon named by a slot expression.
pub fn slot_expr(e: SlotExpr) -> ReadSet {
    match e {
        SlotExpr::This | SlotExpr::Active(_) => ReadSet::st(BOARD),
        // The records cards keep of Pokémon are lasting entries.
        SlotExpr::Recorded => ReadSet::st(BOARD | EFFECTS),
        SlotExpr::Picked | SlotExpr::Attached | SlotExpr::CausePokemon | SlotExpr::Chosen(_) => ReadSet::st(ANY),
    }
}

/// The cards of a zone.
pub fn zone(z: ZoneRef) -> ReadSet {
    match z.1 {
        Zone::Deck | Zone::Hand | Zone::Discard => ReadSet::st(ZONES),
        Zone::Stadium => ReadSet::st(BOARD),
        Zone::Attached(e) | Zone::AttachedEnergy(e) | Zone::Tools(e) => ReadSet::st(BOARD).or(slot_expr(e)),
        Zone::Scratch(_) => ReadSet::st(ANY),
    }
}

pub fn slot_sel(s: &SlotSel) -> ReadSet {
    match s {
        SlotSel::One(e) => ReadSet::st(BOARD).or(slot_expr(*e)),
        SlotSel::Bench(_) | SlotSel::Pokemon(_) | SlotSel::PokemonBenchFirst(_) => ReadSet::st(BOARD),
        SlotSel::Filtered(inner, sp) => slot_sel(inner).or(slot_pred(sp)),
        SlotSel::Cancelable(inner) => slot_sel(inner),
    }
}

/// A predicate on a Pokémon in play (the spot itself is read by the caller: the board).
pub fn slot_pred(sp: &SlotPred) -> ReadSet {
    match sp {
        SlotPred::Any => ReadSet::NONE,
        SlotPred::Not(q) => slot_pred(q),
        SlotPred::All(qs) | SlotPred::OneOf(qs) => all(qs, slot_pred),
        SlotPred::Damaged => ReadSet::st(COUNTERS),
        SlotPred::HasCondition | SlotPred::Condition(_) => ReadSet::st(CONDITIONS),
        SlotPred::IsActive
        | SlotPred::IsBench
        | SlotPred::CardBelowThis(_)
        | SlotPred::Holder
        | SlotPred::Tag(_)
        | SlotPred::RuleBox
        | SlotPred::Basic
        | SlotPred::StageIs(_)
        | SlotPred::Evolution
        | SlotPred::IsThisPokemon
        | SlotPred::Named(_)
        | SlotPred::HasEnergy
        | SlotPred::HasSpecialEnergy
        | SlotPred::HasEnergyNamed(_)
        | SlotPred::Evolved => ReadSet::st(BOARD),
        // The side of the card's owner: where the card is (a zone's owner).
        SlotPred::OnMySide => ReadSet::st(BOARD | ZONES),
        SlotPred::Top(q) | SlotPred::HasCard(q) | SlotPred::AnyTool(q) => ReadSet::st(BOARD).or(pred(q)),
        SlotPred::Recorded => ReadSet::st(BOARD | EFFECTS),
        SlotPred::TypeIs(_) => ReadSet::st(BOARD).or(ReadSet::fact(fact::TYPES)),
        SlotPred::Provides(_) | SlotPred::NoEnergyProvided => ReadSet::st(BOARD).or(ReadSet::fact(fact::PROVIDES)),
        SlotPred::HasAbility | SlotPred::HasAbilityNamed(_) => ReadSet::st(BOARD).or(ReadSet::fact(fact::ABILITY)),
        SlotPred::RemainingHpAtMost(_) => ReadSet::st(BOARD | COUNTERS).or(ReadSet::fact(fact::HP)),
        // `Slot::entered_turn`: a "this turn" record.
        SlotPred::PlayedThisTurn | SlotPred::HealedThisTurn => ReadSet::st(ANY),
    }
}

pub fn num(n: &Num) -> ReadSet {
    match n {
        Num::Lit(_) => ReadSet::NONE,
        Num::ZoneSize(z) => zone(*z),
        Num::CardCount(z, p) | Num::DistinctTypes(z, p) | Num::OthersCount(z, p) => zone(*z).or(pred(p)),
        Num::BenchCount(_) | Num::OpenBench(_) => ReadSet::st(BOARD),
        Num::DamageDone => ReadSet::st(COUNTERS),
        Num::PrizesLeft(_) | Num::PrizesTaken(_) => ReadSet::st(ZONES),
        Num::DamageOn(e) => ReadSet::st(COUNTERS).or(slot_expr(*e)),
        Num::Turn => ReadSet::st(TURN),
        Num::Add(a, b) | Num::Sub(a, b) | Num::Mul(a, b) | Num::Min(a, b) | Num::Max(a, b) => num(a).or(num(b)),
        Num::If(c, a, b) => cond(c).or(num(a)).or(num(b)),
        Num::KnownCopies(..) => ReadSet::st(BOARD | ZONES),
        Num::InPlayCount(_, _, p) => ReadSet::st(BOARD).or(pred(p)),
        Num::SlotCount(sel, sp) => slot_sel(sel).or(slot_pred(sp)),
        Num::EnergyOn(sel, unit) => slot_sel(sel).or(ReadSet::st(BOARD)).or(match unit {
            EnergyUnit::SpecialEnergyCards => ReadSet::NONE,
            EnergyUnit::Provided(_) | EnergyUnit::ProvidedUnits | EnergyUnit::MatchingEntries(_) => ReadSet::fact(fact::PROVIDES),
        }),
        Num::RetreatCostColorless(_) => ReadSet::st(BOARD).or(ReadSet::fact(fact::RETREAT)),
        Num::ToolCount(sel) => ReadSet::st(BOARD).or(slot_sel(sel)),
        Num::DamageSum(sel, sp) => ReadSet::st(COUNTERS).or(slot_sel(sel)).or(slot_pred(sp)),
        // The attack being used, the program's registers and coins, a "last turn" record.
        Num::PrintedCost | Num::RegCount(_) | Num::Heads | Num::DamageTakenLastTurn(_) | Num::PrizesTakenLastTurn(_) | Num::CostNow | Num::Last => ReadSet::st(ANY),
    }
}

pub fn cond(c: &Cond) -> ReadSet {
    match c {
        Cond::Printed(c) | Cond::Not(c) => cond(c),
        Cond::True | Cond::False => ReadSet::NONE,
        Cond::All(cs) | Cond::Any(cs) => all(cs, cond),
        Cond::Cmp(a, _, b) => num(a).or(num(b)),
        Cond::Nonempty(z, p) | Cond::NonemptyOther(z, p) => zone(*z).or(pred(p)),
        Cond::ZoneIs(z, _) => zone(*z),
        Cond::BenchSpace(_) | Cond::HasEvolutionInPool(_) => ReadSet::st(BOARD),
        Cond::IsActive(e) => ReadSet::st(BOARD).or(slot_expr(*e)),
        Cond::Slot(e, sp) => ReadSet::st(BOARD).or(slot_expr(*e)).or(slot_pred(sp)),
        Cond::AnySlot(sel, sp) | Cond::AllSlots(sel, sp) => slot_sel(sel).or(slot_pred(sp)),
        // The counters a Special Condition's rule places are no class (`Slot::poison_damage` and the like).
        Cond::WouldChangeConditions(..) => ReadSet::st(ANY),
        Cond::TypesShared(a, b) => slot_sel(a).or(slot_sel(b)).or(ReadSet::fact(fact::TYPES)),
        // The usage records (the holder of a player's record is the frame's player, a Pokémon's the card).
        Cond::Used(u) => ReadSet::st(EFFECTS | BOARD).or(match u.by {
            super::ops::state::UsedBy::Picked => ReadSet::st(ANY),
            super::ops::state::UsedBy::Copy | super::ops::state::UsedBy::Player => ReadSet::NONE,
        }),
        Cond::LastCardInHand(_) => ReadSet::st(ZONES),
        Cond::KnownCopies { .. } | Cond::AllNamesKnown { .. } => ReadSet::st(BOARD | ZONES),
        Cond::InPlay(_, _, p) | Cond::InPlayAny(_, _, p) | Cond::StadiumInPlay(p) => ReadSet::st(BOARD).or(pred(p)),
        Cond::AbilityBlocked => ReadSet::fact(fact::ABILITY),
        Cond::ToolBlocked => ReadSet::fact(fact::TOOLS_OFF),
        // The program's state, the effect in progress, the "this turn" / "last turn" records, the Prize cards' faces, the
        // evolution rules (checked reads over everything the rules of evolving read).
        Cond::AttackFlag(_)
        | Cond::Chosen(_)
        | Cond::PlayedThisTurn(..)
        | Cond::ThisMovedToActive
        | Cond::Done
        | Cond::DidAll
        | Cond::CanEvolveSkippingStage1
        | Cond::TrainerViaAttack
        | Cond::Attached
        | Cond::KnockedOutLastTurn { .. }
        | Cond::FaceDownPrize(_)
        | Cond::AttackerOfKnockOut { .. }
        | Cond::AttackedLastTurn { .. }
        | Cond::CanEvolveBasic(_) => ReadSet::st(ANY),
    }
}

/// Where a declaration of this origin is in place (`passive::locate`): a card in play or attached, the Stadium in play;
/// a card's rule and a Trainer's effect anywhere, owned by its zone's owner.
pub fn located(origin: RuleSource) -> ReadSet {
    match origin {
        RuleSource::Ability | RuleSource::Tool | RuleSource::Energy | RuleSource::Stadium => ReadSet::st(BOARD),
        RuleSource::TrainerEffect | RuleSource::CardRule => ReadSet::st(BOARD | ZONES),
    }
}

/// Whether the declaration's effect is off (`passive::blocked`): an Ability's "has no Ability", a Tool's "Tools have no
/// effect".
pub fn blocked(origin: RuleSource) -> ReadSet {
    match origin {
        RuleSource::Ability => ReadSet::fact(fact::ABILITY),
        RuleSource::Tool => ReadSet::fact(fact::TOOLS_OFF),
        RuleSource::Energy | RuleSource::Stadium | RuleSource::TrainerEffect | RuleSource::CardRule => ReadSet::NONE,
    }
}

/// Whether an Ability's continuous effect reaches the Pokémon (`passive::ability_effect_reaches`).
pub fn reaches(origin: RuleSource) -> ReadSet {
    match origin {
        RuleSource::Ability => ReadSet::fact(fact::REACH),
        RuleSource::Tool | RuleSource::Energy | RuleSource::Stadium | RuleSource::TrainerEffect | RuleSource::CardRule => ReadSet::NONE,
    }
}

fn cost_change(c: &CostChange) -> ReadSet {
    match c {
        CostChange::Reduce(n) => num(n),
        CostChange::Add(_) | CostChange::Free | CostChange::IgnoreColorless | CostChange::SetCost(_) | CostChange::AnyOne => ReadSet::NONE,
    }
}

/// What the reader of a derived fact's modifier reads for one declaration (the readers in `passive.rs` the
/// `derived::compute_*` call); `None` for a modifier no derived fact reads.
pub fn fact_modifier(ps: &Passive) -> Option<ReadSet> {
    let o = ps.origin;
    let common = located(o).or(blocked(o)).or(reaches(o));
    Some(match &ps.modifier {
        Modifier::HpBonus(_) => common,
        Modifier::HpMod(h) => common.or(slot_pred(&h.subject)).or(cond(&h.guard)).or(num(&h.amount)),
        Modifier::TypeOverride(t) => common.or(slot_pred(&t.subject)),
        // Read for the Energy card on the spot, whatever its origin (`passive::provides_energy_into`).
        Modifier::ProvidesEnergy(pe) => ReadSet::st(BOARD).or(all(pe.entries, |e| slot_pred(&e.when))),
        Modifier::ProvidesEnergyBoost(b) => located(o).or(blocked(o)).or(ReadSet::st(BOARD)).or(pred(&b.energy)),
        Modifier::AttackCost(c) => common.or(ReadSet::st(BOARD)).or(slot_pred(&c.subject)).or(cond(&c.guard)).or(cost_change(&c.change)),
        Modifier::RetreatCost(c) => common.or(ReadSet::st(BOARD)).or(slot_pred(&c.subject)).or(cond(&c.guard)).or(cost_change(&c.change)),
        Modifier::GrantAttacks(_) => located(o).or(blocked(o)).or(ReadSet::st(BOARD)),
        Modifier::BenchAttacks(_) => blocked(o).or(ReadSet::st(BOARD)),
        // "Pokémon Tools have no effect" (`derived::compute_tools_off`): where the card is.
        Modifier::Prevent(p) if p.what == PreventWhat::ToolEffects => located(o),
        Modifier::ActiveLock(l) => active_lock(o, l),
        Modifier::AbilityLock(l) => ability_lock(o, l),
        Modifier::Reflip(_)
        | Modifier::DamageDealt(_)
        | Modifier::DamageTaken(_)
        | Modifier::Prevent(_)
        | Modifier::BlockUse(_)
        | Modifier::SurviveOnTen(_)
        | Modifier::PrizeAdjust(_)
        | Modifier::CheckupDamage(_)
        | Modifier::AttackFlags(_)
        | Modifier::StatOverride(_)
        | Modifier::Recover(_)
        | Modifier::KeepConditions(_)
        | Modifier::AttachGuard(_)
        | Modifier::BenchSize(_)
        | Modifier::AttackTwice(_)
        | Modifier::NextTurnBonus(_)
        | Modifier::Permit(_)
        // Weakness and Resistance are read by the damage calculation only (not memoized).
        | Modifier::WeaknessOverride(_) => return None,
    })
}

fn lock_while(w: &LockWhile) -> ReadSet {
    match w {
        LockWhile::Active | LockWhile::HasTool => ReadSet::st(BOARD),
        // No event while a fact is read (`passive::active_lock_owner` passes none): never the source.
        LockWhile::CardIsSource => ReadSet::NONE,
        LockWhile::Unless(c) => cond(c),
    }
}

/// An Active-Spot Ability lock in the lock pass (`passive::active_lock_applies`, `lock_covers`, `lock_beats`): where the
/// source and the card are, the take-hold stamps (the turn player's first when they take hold together), the source's
/// own Ability, the continuous-effect view.
fn active_lock(o: RuleSource, l: &ActiveLockSpec) -> ReadSet {
    lock_pass(o).or(slot_pred(&l.locked)).or(pred(&l.except)).or(all(l.while_, lock_while))
}

fn ability_lock(o: RuleSource, l: &AbilityLockSpec) -> ReadSet {
    let locker = match l.locker {
        Locker::BenchOfEitherSide | Locker::InPlayEitherSide | Locker::StadiumInPlay => ReadSet::st(BOARD),
    };
    let probe = match l.probe {
        LockerProbe::Generic | LockerProbe::OwnPower(_) | LockerProbe::Stadium => ReadSet::fact(fact::ABILITY),
    };
    lock_pass(o).or(locker).or(probe).or(pred(&l.card)).or(slot_pred(&l.slot)).or(pred(&l.missing))
}

/// What every lock of the lock pass reads: where the cards are (a card in no list is judged by its printed data), the
/// stamps, the turn player (`lock_turn_order`), the other locks (`power_blocked` again), the continuous-effect view.
fn lock_pass(o: RuleSource) -> ReadSet {
    ReadSet::st(BOARD | ZONES | EFFECTS | TURN).or(ReadSet::fact(fact::ABILITY)).or(located(o)).or(ReadSet::fact(fact::REACH))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every declaration a derived fact's computation reads (the families of `derived::FACT_FAMILIES`, the Ability locks
    /// of the lock pass) is classified; those whose reading reaches state no class names (`ANY`: their fact is keyed by
    /// every write in a game holding them) are listed, and stay a minority (speed batch L3).
    #[test]
    fn every_fact_declaration_is_classified() {
        let mut total = 0;
        let mut any: Vec<&str> = Vec::new();
        for spec in crate::cards::registry::SPECS.iter() {
            for ps in spec.passives {
                let read = match crate::decl::modifier_family(&ps.modifier) {
                    Some(f) => crate::derived::FACT_FAMILIES.iter().any(|(fm, _)| *fm == f),
                    None => false,
                } || matches!(ps.modifier, Modifier::ActiveLock(_) | Modifier::AbilityLock(_));
                if !read {
                    continue;
                }
                let r = fact_modifier(ps).unwrap_or_else(|| panic!("{}: a fact's declaration with no classification", spec.class));
                total += 1;
                if r.state & ANY != 0 {
                    any.push(spec.class);
                }
            }
        }
        println!("{} fact declarations, {} read state no class names: {:?}", total, any.len(), any);
        assert!(total > 0 && any.len() * 4 < total, "{} of {} fact declarations read anything", any.len(), total);
    }
}
