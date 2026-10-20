//! Retreating (APR A-03, "Steps for Retreating"): the turn action's rules (once per turn, not Asleep or Paralyzed),
//! then step 2, the Retreat Cost paid, then step 3, the Benched Pokémon chosen and sent to the Active Spot by the
//! ChangeActive event (`engine::change_active`, kind `Retreat`), which carries the checks ("can't retreat" locks, the
//! Pokémon's lasting "can't retreat") and the consequences (only the Pokémon moving to the Bench loses its effects and
//! Special Conditions: id356, id891). The action names no Benched Pokémon: it is chosen after the cost is paid (user
//! decision D5, batch 10). A retreat that no Benched Pokémon could complete is not started: the ChangeActive's checks
//! run against every Benched Pokémon before the cost is paid.

use crate::effects::*;
use crate::energy;
use crate::engine::change_active::{self, ActiveChecks, ChangeActiveView};
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

/// Where a retreat waits: for the Energy cards paying the cost (step 2), or for the Benched Pokémon (step 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetreatStep {
    Pay,
    Choose,
}

#[derive(Clone, Copy, Debug)]
pub struct RetreatCont {
    pub p: u8,
    pub move_to: ListRef,
    pub step: RetreatStep,
}

/// The ChangeActive a retreat of player `p` to the Benched Pokémon in spot `bench` makes.
pub fn retreat_change(g: &Game, p: usize, bench: SlotId) -> ChangeActiveView {
    let cause = crate::cause::Cause::rule(crate::cause::RuleWhich::Retreat, p as u8);
    ChangeActiveView::of(g, p, Some(bench), crate::spec::event::ActiveChange::Retreat, cause)
}

/// The Benched Pokémon (their spots, in Bench order) player `p`'s retreat may send to the Active Spot: every one whose
/// ChangeActive passes its checks (`change_active::check_with`: on the game for execution, on legality's context for
/// legality, so the two can't drift). `first`: stop at the first one (legality asks whether there is one). The error
/// is the refusal of the first Benched Pokémon when none is allowed.
pub fn allowed<C: ActiveChecks + ?Sized>(c: &mut C, p: usize, first: bool) -> R<SVec<SlotId, MAX_BENCH>> {
    let mut out = SVec::new();
    let mut refusal: &'static str = "INVALID_TARGET";
    let bench = c.game().st.players[p].bench;
    for &b in bench.iter() {
        if c.game().st.slot_pokemon(p, b).is_none() {
            continue;
        }
        let v = {
            let g = c.game();
            change_active::view(g, &retreat_change(g, p, b))
        };
        match change_active::check_with(c, &v)? {
            None => {
                out.push(b);
                if first {
                    break;
                }
            }
            Some(code) if out.is_empty() && refusal == "INVALID_TARGET" => refusal = code,
            Some(_) => {}
        }
    }
    if out.is_empty() {
        crate::bail!(refusal);
    }
    Ok(out)
}

fn energy_cards(map: &EnergyMap) -> Vec<CardId> {
    let mut v = Vec::new();
    for e in map.iter() {
        for _ in 0..energy::unit_count(e.provides.as_slice()) {
            v.push(e.card);
        }
    }
    v
}

/// Check only: the turn action's rules for retreating (a Benched Pokémon, Special Conditions, once per turn). The
/// ChangeActive's own checks (`change_active::check_with`: the locks, the Pokémon's lasting "can't retreat") come with
/// it ([`allowed`]), on the game (execution) or legality's context.
pub fn can_retreat(g: &Game, p: usize, ignore_status_conditions: bool) -> R {
    let pl = &g.st.players[p];
    if !pl.bench.iter().any(|&b| g.st.slot_pokemon(p, b).is_some()) {
        crate::bail!("INVALID_TARGET");
    }
    let sp = g.st.slot(p, pl.active).special_conditions;
    if (sp.contains(&(SpecialCondition::Paralyzed as u8)) || sp.contains(&(SpecialCondition::Asleep as u8))) && !ignore_status_conditions {
        crate::bail!("BLOCKED_BY_SPECIAL_CONDITION");
    }
    if pl.retreated_turn == g.st.turn {
        crate::bail!("RETREAT_ALREADY_USED");
    }
    Ok(())
}

/// The checked reads of a retreat: the effective cost
/// and, when the cost isn't empty, the Energy the Active Pokémon provides
/// (the map is empty otherwise, as no read is run).
pub fn retreat_read(g: &mut Game, p: usize) -> R<(Cost, EnergyMap)> {
    let active = g.st.players[p].active;
    let cost = crate::derived::retreat_cost(g, p)?;
    if cost.is_empty() {
        return Ok((cost, SVec::new()));
    }
    let map = crate::derived::provided_energy(g, p, SlotRef::new(p, active))?;
    Ok((cost, map))
}

pub fn reducer(g: &mut Game, id: EffId) -> R {
    let (p, ignore, move_to) = match *g.e(id) {
        Effect::Retreat { p, ignore_status_conditions, move_retreat_cost_to } => (p as usize, ignore_status_conditions, move_retreat_cost_to),
        _ => return Ok(()),
    };
    can_retreat(g, p, ignore)?;
    // Step 3 can be done: some Benched Pokémon can be sent to the Active Spot (checked before the cost is paid).
    allowed(g, p, true)?;
    let active = g.st.players[p].active;
    let (cost, map) = retreat_read(g, p)?;
    if cost.is_empty() {
        return choose(g, p, move_to);
    }
    if !energy::check_enough_energy(map.as_slice(), cost.as_slice()) {
        crate::bail!("NOT_ENOUGH_ENERGY");
    }
    // A table that asks every payment (`TableRules::ask_single_way_payments`) asks also the payments made in one way below.
    let single_way_paid = !g.st.rules.ask_single_way_payments;
    if single_way_paid && energy::check_exact_energy(map.as_slice(), cost.as_slice()) {
        let cards = energy_cards(&map);
        pay(g, p, active, &cards, move_to)?;
        return choose(g, p, move_to);
    }
    // Interchangeable Energy cards (copies of one card, each providing the same) paid in one way only: which of them is
    // discarded can't change the game, nothing is asked; different cards are the player's choice (APR A-03 step 2). Paid
    // in two ways, as many cards as the cost or until the units are met (id2127, n1652: "you could discard either 1 or 2
    // Double Turbo Energy"; never more cards than the cost), the retreat asks (user decision 2026-10-13; the cards are
    // counted, not the entries of the Energy map).
    let only = if single_way_paid { energy::only_payment(map.as_slice(), cost.as_slice(), |c| g.st.cards[c as usize].def) } else { None };
    if let Some(cards) = only {
        pay(g, p, active, cards.as_slice(), move_to)?;
        return choose(g, p, move_to);
    }
    let map = g.stash_energy(map);
    g.prompt(
        p,
        PromptPurpose::DiscardEnergy,
        PromptKind::ChooseEnergy { energy: map, cost, allow_cancel: true },
        crate::prompts::Provenance::NONE,
        Cont::Retreat(RetreatCont { p: p as u8, move_to, step: RetreatStep::Pay }),
    );
    Ok(())
}

/// Step 3, the cost paid: the player has retreated this turn and chooses the Benched Pokémon to send to the Active
/// Spot among those its ChangeActive allows (asked like the promotion, also when one is allowed; the outcome is the
/// Pokémon that becomes the Active Pokémon, so the purpose is `SwitchPokemon`). When none is allowed any more (a
/// guard: no check of a pool card reads what paying the cost changes) the retreat ends with the cost paid.
fn choose(g: &mut Game, p: usize, move_to: ListRef) -> R {
    g.st.players[p].retreated_turn = g.st.turn;
    if g.st.active_pokemon(p).is_none() {
        return Ok(());
    }
    let Ok(ok) = allowed(g, p, false) else { return Ok(()) };
    let mut blocked: TargetList = SVec::new();
    for (i, b) in g.st.players[p].bench.iter().enumerate() {
        if !ok.contains(b) {
            blocked.push(CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, i as u8));
        }
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    g.prompt(
        p,
        PromptPurpose::SwitchPokemon,
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        crate::prompts::Provenance::NONE,
        Cont::Retreat(RetreatCont { p: p as u8, move_to, step: RetreatStep::Choose }),
    );
    Ok(())
}

pub fn resume(g: &mut Game, rc: RetreatCont, res: Res) -> R {
    let p = rc.p as usize;
    match rc.step {
        RetreatStep::Pay => {
            let energy = match res {
                Res::Energy(e) => e,
                _ => return Ok(()),
            };
            if g.st.active_pokemon(p).is_none() {
                return Ok(());
            }
            let cards: Vec<CardId> = energy.iter().copied().collect();
            let active = g.st.players[p].active;
            pay(g, p, active, &cards, rc.move_to)?;
            choose(g, p, rc.move_to)
        }
        RetreatStep::Choose => {
            let sel = res.slots();
            if sel.len() != 1 || sel[0].p as usize != p || g.st.players[p].bench_index_of(sel[0].s).is_none() {
                crate::bail!("ILLEGAL_ACTION");
            }
            if g.st.active_pokemon(p).is_none() {
                return Ok(());
            }
            let c = retreat_change(g, p, sel[0].s);
            change_active::check(g, &c)?;
            change_active::produce(g, c)
        }
    }
}

/// The Retreat Cost paid: the chosen Energy cards leave play from the Active Pokémon for their owner's discard pile (a
/// LeavePlay of attached cards by the rule, user decision D1; never refused).
fn pay(g: &mut Game, p: usize, active: crate::state::SlotId, cards: &[CardId], to: ListRef) -> R {
    let zone = crate::engine::knockout::zone_of(to);
    crate::engine::knockout::leave_play_cards_by_rule(g, SlotRef::new(p, active), cards, zone, crate::cause::Cause::rule(crate::cause::RuleWhich::Retreat, p as u8))
}

/// The Active Pokémon's printed Retreat
/// Cost, plus one [C] per point of retreatCostIncreaseNextTurn (Rillaboom's
/// Drum Beating), which is part of the base cost so a "no Retreat Cost" effect
/// still wins.
pub fn check_retreat_cost_base(g: &Game, p: usize) -> Cost {
    let mut cost: Cost = SVec::new();
    if let Some(c) = g.st.active_pokemon(p) {
        for &t in g.st.cdef(c).retreat {
            cost.push(t);
        }
    }
    cost
}
