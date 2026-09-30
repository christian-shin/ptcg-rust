//! `check-effect.ts`: knock-outs, prizes, new Active, winner; plus the
//! check-state reducer for cost and energy checks.

use crate::effects::*;
use crate::engine::phase::end_game;
use crate::game::{Cont, Game, OnComplete, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckStage {
    /// Resume after KO number `idx` (its effect is `ko_fx`).
    AfterKo,
    AfterBenchSize,
    AfterPrize,
    AfterActive,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PrizeGroup {
    pub destination: Option<ListRef>,
    pub count: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct CheckFrame {
    pub stage: CheckStage,
    pub oc: OnComplete,
    pub kos: SVec<SlotRef, 16>,
    pub idx: u8,
    pub ko_fx: EffId,
    pub groups: [SVec<(ListRef, i32), 4>; 2],
    /// Prize prompts to open: (player, count, destination).
    pub prize_prompts: SVec<(u8, i32, ListRef), 4>,
    pub active_prompts: SVec<u8, 2>,
}

pub fn check_state(g: &mut Game, oc: OnComplete) -> R {
    if !matches!(g.st.phase, GamePhase::PlayerTurn | GamePhase::Attack | GamePhase::BetweenTurns) {
        return on_complete(g, oc);
    }
    let kos = find_ko_pokemons(g)?;
    let f = CheckFrame {
        stage: CheckStage::AfterKo,
        oc,
        kos,
        idx: 0,
        ko_fx: 0,
        groups: [SVec::new(), SVec::new()],
        prize_prompts: SVec::new(),
        active_prompts: SVec::new(),
    };
    ko_loop(g, f)
}

pub fn on_complete(g: &mut Game, oc: OnComplete) -> R {
    match oc {
        OnComplete::None => Ok(()),
        OnComplete::AfterEndTurn { p } => crate::engine::phase::after_end_turn(g, p as usize),
        OnComplete::InitNextTurn => {
            if g.st.phase != GamePhase::Finished {
                crate::engine::phase::init_next_turn(g)
            } else {
                Ok(())
            }
        }
    }
}

/// `CheckHpEffect`: constructing it resets `hpBonus`; returns the final HP.
pub fn check_hp(g: &mut Game, p: usize, s: SlotId) -> R<i32> {
    let card = g.st.slot_pokemon(p, s);
    if card.is_some() {
        g.st.players[p].slots[s as usize].hp_bonus = 0;
    }
    g.run_fx(Effect::CheckHp { p: p as u8, target: SlotRef::new(p, s), card })?;
    Ok(hp_of(g, p, s, card))
}

pub fn hp_of(g: &Game, p: usize, s: SlotId, card: Option<CardId>) -> i32 {
    match card {
        Some(c) => g.st.cdef(c).hp + g.st.slot(p, s).hp_bonus,
        None => 0,
    }
}

fn find_ko_pokemons(g: &mut Game) -> R<SVec<SlotRef, 16>> {
    let mut out = SVec::new();
    for p in 0..2 {
        for s in g.st.players[p].in_play().iter() {
            let hp = check_hp(g, p, *s)?;
            if g.st.slot(p, *s).damage >= hp {
                out.push(SlotRef::new(p, *s));
            }
        }
    }
    Ok(out)
}

fn add_prize(f: &mut CheckFrame, taker: usize, destination: ListRef, count: i32) {
    let groups = &mut f.groups[taker];
    if let Some(gr) = groups.as_mut_slice().iter_mut().find(|(d, _)| *d == destination) {
        gr.1 += count;
    } else {
        groups.push((destination, count));
    }
}

fn ko_loop(g: &mut Game, mut f: CheckFrame) -> R {
    while (f.idx as usize) < f.kos.len() {
        let t = *f.kos.get(f.idx as usize).unwrap();
        let id = g.new_fx(Effect::KnockOut { p: t.p, target: t, prize_count: 1, prize_destination: None, attack: None });
        g.reduce_effect(id)?;
        f.ko_fx = id;
        if g.has_prompts() {
            f.stage = CheckStage::AfterKo;
            g.wait_prompt(Cont::CheckState(f));
            return Ok(());
        }
        finish_ko(g, &mut f);
    }
    after_kos(g, f)
}

fn finish_ko(g: &mut Game, f: &mut CheckFrame) {
    let id = f.ko_fx;
    if !g.prevented(id) {
        if let Effect::KnockOut { p, prize_count, prize_destination, .. } = *g.e(id) {
            let taker = 1 - p as usize;
            let dest = prize_destination.unwrap_or(ListRef::Hand(taker as u8));
            add_prize(f, taker, dest, prize_count);
        }
    }
    g.release_fx(id);
    f.idx += 1;
}

fn after_kos(g: &mut Game, mut f: CheckFrame) -> R {
    let (e, _) = g.run_fx(Effect::CheckTableState { bench_sizes: [5, 5] })?;
    // cannotBeSpecialConditionedNextTurn clearing: not modeled.
    let sizes = match e {
        Effect::CheckTableState { bench_sizes } => bench_sizes,
        _ => [5, 5],
    };
    handle_bench_size_change(g, sizes);
    if g.has_prompts() {
        f.stage = CheckStage::AfterBenchSize;
        g.wait_prompt(Cont::CheckState(f));
        return Ok(());
    }
    after_bench_size(g, f)
}

fn after_bench_size(g: &mut Game, mut f: CheckFrame) -> R {
    if g.st.phase == GamePhase::Finished {
        return Ok(());
    }
    f.prize_prompts = choose_prize_cards(g, &mut f)?;
    f.idx = 0;
    prize_loop(g, f)
}

fn prize_loop(g: &mut Game, mut f: CheckFrame) -> R {
    if (f.idx as usize) < f.prize_prompts.len() {
        let (p, count, dest) = *f.prize_prompts.get(f.idx as usize).unwrap();
        f.idx += 1;
        let pl = p as usize;
        let message = if dest == ListRef::Discard(p) { "CHOOSE_PRIZE_CARD_TO_DISCARD" } else { "CHOOSE_PRIZE_CARD" };
        let id = g.player_id(pl);
        g.prompt(
            id,
            message,
            PromptKind::ChoosePrize {
                count: count as u8,
                blocked: SVec::new(),
                use_opponent_prizes: false,
                allow_cancel: false,
                is_secret: true,
                destination: Some(dest),
            },
            Cont::TakePrizes { p, destination: dest },
        );
        if g.has_prompts() {
            f.stage = CheckStage::AfterPrize;
            g.wait_prompt(Cont::CheckState(f));
            return Ok(());
        }
        return prize_loop(g, f);
    }
    if g.st.phase == GamePhase::Finished {
        return on_complete(g, f.oc);
    }
    let prizes_taken = g.st.players.iter().any(|pl| pl.prizes[..pl.prize_count as usize].iter().all(|l| l.is_empty()));
    if prizes_taken {
        return check_winner(g, f.oc);
    }
    f.active_prompts = SVec::new();
    for p in 0..2 {
        let pl = &g.st.players[p];
        let has_active = !pl.slots[pl.active as usize].cards.is_empty();
        let has_bench = pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty());
        if !has_active && has_bench {
            f.active_prompts.push(p as u8);
        }
    }
    f.idx = 0;
    active_loop(g, f)
}

fn active_loop(g: &mut Game, mut f: CheckFrame) -> R {
    if (f.idx as usize) < f.active_prompts.len() {
        let p = *f.active_prompts.get(f.idx as usize).unwrap();
        f.idx += 1;
        let id = g.player_id(p as usize);
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        g.prompt(
            id,
            "CHOOSE_NEW_ACTIVE_POKEMON",
            PromptKind::ChoosePokemon {
                player_type: PlayerType::BottomPlayer,
                slots,
                min: 1,
                max: 1,
                allow_cancel: false,
                blocked: SVec::new(),
            },
            Cont::ChooseActive { p },
        );
        if g.has_prompts() {
            f.stage = CheckStage::AfterActive;
            g.wait_prompt(Cont::CheckState(f));
            return Ok(());
        }
        return active_loop(g, f);
    }
    check_winner(g, f.oc)?;
    g.st.bench_size_change_handled = false;
    Ok(())
}

pub fn resume(g: &mut Game, mut f: CheckFrame) -> R {
    match f.stage {
        CheckStage::AfterKo => {
            finish_ko(g, &mut f);
            ko_loop(g, f)
        }
        CheckStage::AfterBenchSize => after_bench_size(g, f),
        CheckStage::AfterPrize => prize_loop(g, f),
        CheckStage::AfterActive => active_loop(g, f),
    }
}

pub fn take_prizes_cont(g: &mut Game, p: u8, destination: ListRef, res: Res) -> R {
    if let Res::Prizes(ix) = res {
        take_specific_prizes(g, p as usize, ix.as_slice(), destination, false)?;
    }
    Ok(())
}

pub fn choose_active_cont(g: &mut Game, p: u8, res: Res) -> R {
    let p = p as usize;
    let sel = res.slots();
    if sel.len() != 1 {
        crate::bail!("ILLEGAL_ACTION");
    }
    let bench_index = g.st.players[p].bench_index_of(sel[0].s);
    let active = g.st.players[p].active;
    let bi = match bench_index {
        Some(i) if sel[0].p as usize == p && g.st.slot(p, active).cards.is_empty() => i,
        _ => crate::bail!("ILLEGAL_ACTION"),
    };
    let pl = &mut g.st.players[p];
    let new_active = pl.bench.as_slice()[bi];
    pl.bench.as_mut_slice()[bi] = pl.active;
    pl.active = new_active;
    if let Some(c) = g.st.slot_pokemon(p, new_active) {
        if !g.st.players[p].moved_to_active_this_turn.contains(&c) {
            g.st.players[p].moved_to_active_this_turn.push(c);
        }
        g.st.cards[c as usize].moved_to_active_this_turn = true;
        g.run_fx(Effect::MovedToActive { p: p as u8, card: c })?;
    }
    Ok(())
}

fn opponent_has_no_pokemon_in_play(g: &Game, taker: usize) -> bool {
    let o = &g.st.players[1 - taker];
    if !o.slots[o.active as usize].cards.is_empty() {
        return false;
    }
    !o.bench.iter().any(|b| !o.slots[*b as usize].cards.is_empty())
}

fn auto_take_prize_cards(g: &mut Game, p: usize, count: usize, destination: ListRef) -> R {
    let ix: SVec<u8, 6> = {
        let mut v = SVec::new();
        for i in 0..g.st.players[p].prize_count {
            if !g.st.players[p].prizes[i as usize].is_empty() && v.len() < count {
                v.push(i);
            }
        }
        v
    };
    if ix.is_empty() {
        return Ok(());
    }
    take_specific_prizes(g, p, ix.as_slice(), destination, false)
}

fn choose_prize_cards(g: &mut Game, f: &mut CheckFrame) -> R<SVec<(u8, i32, ListRef), 4>> {
    let mut prompts = SVec::new();
    for i in 0..2 {
        let groups = f.groups[i];
        for gi in 0..groups.len() {
            let (dest, mut count) = *groups.get(gi).unwrap();
            let left = g.st.players[i].prize_left() as i32;
            if count > 0 && g.st.is_sudden_death {
                end_game(g, if i == 0 { WINNER_P1 } else { WINNER_P2 });
                return Ok(SVec::new());
            }
            if count >= left && left > 0 {
                auto_take_prize_cards(g, i, left as usize, dest)?;
                end_game(g, if i == 0 { WINNER_P1 } else { WINNER_P2 });
                return Ok(SVec::new());
            }
            if count > 0 && opponent_has_no_pokemon_in_play(g, i) {
                auto_take_prize_cards(g, i, count as usize, dest)?;
                continue;
            }
            if count > left {
                count = left;
            }
            if count > 0 {
                prompts.push((i as u8, count, dest));
            }
        }
    }
    Ok(prompts)
}

/// `TAKE_SPECIFIC_PRIZES`.
pub fn take_specific_prizes(g: &mut Game, p: usize, prizes: &[u8], destination: ListRef, skip_reduce: bool) -> R {
    let mut destination = destination;
    let mut prevented = false;
    if !skip_reduce {
        let mut mask = 0u8;
        for &i in prizes {
            mask |= 1 << i;
        }
        let (e, pd) = g.run_fx(Effect::CheckPrizesDestination { p: p as u8, destination })?;
        let mut draw_dest = destination;
        if !pd {
            if let Effect::CheckPrizesDestination { destination: d, .. } = e {
                draw_dest = d;
            }
        }
        let (e2, pd2) = g.run_fx(Effect::DrawPrizes { p: p as u8, prizes: mask, destination: draw_dest })?;
        prevented = pd2;
        if let Effect::DrawPrizes { destination: d, .. } = e2 {
            destination = d;
        }
    } else {
        destination = ListRef::Hand(p as u8);
    }
    if !prevented {
        for &i in prizes {
            let src = ListRef::Prize(p as u8, i);
            g.move_to(src, destination, None);
            if destination == ListRef::Hand(p as u8) {
                g.st.players[p].prizes_taken += 1;
                g.st.players[p].prizes_taken_this_turn += 1;
            }
        }
    }
    Ok(())
}

pub fn check_winner(g: &mut Game, oc: OnComplete) -> R {
    let mut points = [0; 2];
    for i in 0..2 {
        let pl = &g.st.players[i];
        if pl.slots[pl.active as usize].cards.is_empty() {
            points[1 - i] += 1;
        }
        if pl.prizes[..pl.prize_count as usize].iter().all(|l| l.is_empty()) {
            points[i] += 1;
        }
    }
    if points[0] > 0 && points[1] > 0 {
        // Sudden death is not ported yet; treated as a draw so traces flag it.
        crate::bail!("SUDDEN_DEATH_NOT_PORTED");
    }
    if points[0] + points[1] == 0 {
        return on_complete(g, oc);
    }
    let winner = if points[0] > points[1] {
        WINNER_P1
    } else if points[1] > points[0] {
        WINNER_P2
    } else {
        WINNER_DRAW
    };
    end_game(g, winner);
    on_complete(g, oc)
}

fn handle_bench_size_change(g: &mut Game, sizes: [u8; 2]) {
    if g.st.bench_size_change_handled {
        return;
    }
    for p in 0..2 {
        let size = sizes[p] as usize;
        while g.st.players[p].bench.len() < size {
            let s = g.st.players[p].alloc_slot();
            g.st.players[p].slots[s as usize].is_public = true;
            g.st.players[p].bench.push(s);
        }
        if g.st.players[p].bench.len() == size {
            continue;
        }
        // Shrinking the bench (Area Zero leaving play) is not ported yet.
        let pl = &mut g.st.players[p];
        let mut i = pl.bench.len();
        while i > 0 && pl.bench.len() > size {
            i -= 1;
            let s = pl.bench.as_slice()[i];
            if pl.slots[s as usize].cards.is_empty() {
                pl.bench.remove_at(i);
                pl.free_slot(s);
            }
        }
    }
    g.st.bench_size_change_handled = true;
}

// ---------------------------------------------------------------------------
// checkStateReducer

pub fn check_state_reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::CheckAttackCost { .. } => {
            // attackCostIncreaseWhileActive / ignoreAttackCostCardTypes: not modeled.
            Ok(())
        }
        Effect::CheckRetreatCost { .. } => {
            // zeroRetreatCostNextTurn: not modeled.
            Ok(())
        }
        Effect::CheckProvidedEnergy { source, .. } => {
            let slot = *g.st.slot(source.p as usize, source.s);
            let mut add: SVec<EnergyEntry, 40> = SVec::new();
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e(id) {
                for c in slot.cards.iter() {
                    let d = g.st.cdef(c);
                    if d.is_energy() && !energy_map.iter().any(|e| e.card == c) && !add.iter().any(|e| e.card == c) {
                        let mut provides = SVec::new();
                        for &t in d.provides {
                            provides.push(t);
                        }
                        add.push(EnergyEntry { card: c, provides });
                    }
                }
                // Pokémon-as-energy in `energies` (provides set by the card): not modeled.
            }
            if let Effect::CheckProvidedEnergy { energy_map, .. } = g.e_mut(id) {
                for e in add.iter() {
                    energy_map.push(*e);
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
