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
    /// Resume after KO number `idx` was announced (its effect is `ko_fx`).
    AfterKo,
    /// Resume after Knock Out number `idx` took its Pokémon out of play (effect `ko_fxs[idx]`).
    AfterRemove,
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
    pub kos: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }>,
    /// The Pokémon whose Knock Out was announced in this round (a prevented one stays at 0 HP and is not a new Knock Out).
    pub announced: SVec<CardId, 16>,
    pub idx: u8,
    pub ko_fx: EffId,
    /// The announced Knock Out effects (retained until the Pokémon left play).
    pub ko_fxs: SVec<EffId, 16>,
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
    let mut announced = SVec::new();
    for k in kos.iter() {
        if let Some(c) = g.st.slot_pokemon(k.p as usize, k.s) {
            announced.push(c);
        }
    }
    let f = CheckFrame {
        stage: CheckStage::AfterKo,
        oc,
        kos,
        announced,
        idx: 0,
        ko_fx: 0,
        ko_fxs: SVec::new(),
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

fn find_ko_pokemons(g: &mut Game) -> R<SVec<SlotRef, { crate::state::MAX_SLOT_REFS }>> {
    let mut out = SVec::new();
    for p in 0..2 {
        for s in g.st.players[p].in_play().iter() {
            // forEachPokemon skips slots without a Pokémon card (e.g. a Bench
            // slot holding only an Energy moved there by Team Rocket's Zapdos).
            if g.st.slot_pokemon(p, *s).is_none() {
                continue;
            }
            // In a legality trial an undamaged Pokémon can't be Knocked Out
            // (no effect lowers HP to 0); its HP check is skipped.
            if g.trial && g.st.slot(p, *s).damage == 0 {
                continue;
            }
            let hp = check_hp(g, p, *s)?;
            if g.st.slot(p, *s).damage >= hp {
                out.push(SlotRef::new(p, *s));
            }
        }
    }
    Ok(out)
}

/// `winConditionMet`: `check_winner` would end the game or start a Tiebreaker (a player has no Pokémon in
/// play, an Active spot waiting for a promotion from the Bench does not count, or no Prize cards left).
fn win_condition_met(g: &Game) -> bool {
    (0..2).any(|i| {
        let pl = &g.st.players[i];
        let no_pokemon = pl.slots[pl.active as usize].cards.is_empty() && !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty());
        no_pokemon || pl.prizes[..pl.prize_count as usize].iter().all(|l| l.is_empty())
    })
}

/// `knockOutPendingBeforeWinner`: a Knock Out effect at step 2 (Maractus JTG's Explosive Needle) can put counters
/// on a Pokémon that is then Knocked Out in a further round. Every effect has to resolve before the winner is
/// determined, so that round's Knock Outs and Prizes count before the game ends (rulings 1577, 1584, 1403).
fn knock_out_pending_before_winner(g: &mut Game, f: &CheckFrame) -> R<bool> {
    if !win_condition_met(g) {
        return Ok(false);
    }
    for k in find_ko_pokemons(g)?.iter() {
        let c = g.st.slot_pokemon(k.p as usize, k.s);
        if !c.map_or(false, |c| f.announced.contains(&c)) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn add_prize(f: &mut CheckFrame, taker: usize, destination: ListRef, count: i32) {
    let groups = &mut f.groups[taker];
    if let Some(gr) = groups.as_mut_slice().iter_mut().find(|(d, _)| *d == destination) {
        gr.1 += count;
    } else {
        groups.push((destination, count));
    }
}

/// Every Knock Out is announced while all Pokémon are still in play (a Pokémon Knocked
/// Out at the same time still has its Ability: ruling 1623), then they leave play and
/// the Prizes are counted (R7F-16; Twinleaf `executeCheckState`).
fn ko_loop(g: &mut Game, mut f: CheckFrame) -> R {
    while (f.idx as usize) < f.kos.len() {
        let t = *f.kos.get(f.idx as usize).unwrap();
        let id = g.new_fx(Effect::KnockOut { p: t.p, target: t, prize_count: 1, prize_base: 1, prize_destination: None, attack: None, defer_removal: true });
        g.reduce_effect(id)?;
        f.ko_fx = id;
        if g.has_prompts() {
            f.stage = CheckStage::AfterKo;
            g.wait_prompt(Cont::CheckState(f));
            return Ok(());
        }
        f.ko_fxs.push(id);
        f.idx += 1;
    }
    f.idx = 0;
    remove_loop(g, f)
}

fn remove_loop(g: &mut Game, mut f: CheckFrame) -> R {
    while (f.idx as usize) < f.ko_fxs.len() {
        let id = *f.ko_fxs.get(f.idx as usize).unwrap();
        if !g.prevented(id) {
            crate::engine::game_effect::complete_knock_out(g, id)?;
            if g.has_prompts() {
                f.stage = CheckStage::AfterRemove;
                g.wait_prompt(Cont::CheckState(f));
                return Ok(());
            }
        }
        finish_ko(g, &mut f);
    }
    after_kos(g, f)
}

fn finish_ko(g: &mut Game, f: &mut CheckFrame) {
    let id = *f.ko_fxs.get(f.idx as usize).unwrap();
    if !g.prevented(id) {
        if let Effect::KnockOut { p, prize_count, prize_destination, .. } = *g.e(id) {
            let taker = 1 - p as usize;
            let dest = prize_destination.unwrap_or(ListRef::Hand(taker as u8));
            // Prize reductions (Legacy Energy, Lillie's Pearl, ...) never go below 0 (R7F-13, ruling 1745).
            add_prize(f, taker, dest, prize_count.max(0));
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
                is_secret: !g.st.players[pl].prize_public[0],
                destination: Some(dest),
                face_down_only: false,
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
    // A player has no Prize cards left: the game is decided now (check_winner
    // counts both players' win conditions) unless both took their last Prize card
    // at the same time: then the new Active Pokémon are promoted and their effects
    // resolve before the winner is determined (R7F-12; rulings 820, 1584).
    let taken: [bool; 2] = [0, 1].map(|i: usize| g.st.players[i].prizes[..g.st.players[i].prize_count as usize].iter().all(|l| l.is_empty()));
    if (taken[0] || taken[1]) && !(taken[0] && taken[1]) {
        if knock_out_pending_before_winner(g, &f)? {
            return check_state(g, f.oc);
        }
        return check_winner(g, f.oc);
    }
    f.active_prompts = SVec::new();
    for p in next_turn_player_order(g) {
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
    if knock_out_pending_before_winner(g, &f)? {
        return check_state(g, f.oc);
    }
    check_winner(g, f.oc)?;
    g.st.bench_size_change_handled = false;
    Ok(())
}

pub fn resume(g: &mut Game, mut f: CheckFrame) -> R {
    match f.stage {
        CheckStage::AfterKo => {
            f.ko_fxs.push(f.ko_fx);
            f.idx += 1;
            ko_loop(g, f)
        }
        CheckStage::AfterRemove => {
            finish_ko(g, &mut f);
            remove_loop(g, f)
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

/// The player whose turn would be next takes Prizes first and promotes first
/// when both have Pokémon Knocked Out at the same time (rulings 754, 757).
fn next_turn_player_order(g: &Game) -> [usize; 2] {
    let next = if g.st.active_player == 0 { 1 } else { 0 };
    [next, 1 - next]
}

fn choose_prize_cards(g: &mut Game, f: &mut CheckFrame) -> R<SVec<(u8, i32, ListRef), 4>> {
    let mut prompts = SVec::new();
    let mut took_last_prize = false;
    for i in next_turn_player_order(g) {
        let groups = f.groups[i];
        for gi in 0..groups.len() {
            let (dest, mut count) = *groups.get(gi).unwrap();
            let left = g.st.players[i].prize_left() as i32;
            // Taking the last Prize cards does not end the game here: every effect
            // resolves and check_winner counts both players' win conditions
            // (R7F-12; rulings 234, 820, 1403, 1584).
            if count >= left && left > 0 {
                auto_take_prize_cards(g, i, left as usize, dest)?;
                took_last_prize = true;
                continue;
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
    if took_last_prize {
        return Ok(SVec::new());
    }
    Ok(prompts)
}

/// `TAKE_X_PRIZES(store, state, player, count)` (default options).
pub fn take_x_prizes(g: &mut Game, p: usize, count: i32) -> R {
    let left = g.st.players[p].prize_left() as i32;
    let take = count.min(left);
    if take <= 0 {
        return Ok(());
    }
    if count >= left {
        let mut ix: SVec<u8, 6> = SVec::new();
        for i in 0..g.st.players[p].prize_count {
            if !g.st.players[p].prizes[i as usize].is_empty() && (ix.len() as i32) < take {
                ix.push(i);
            }
        }
        return take_specific_prizes(g, p, ix.as_slice(), ListRef::Hand(p as u8), false);
    }
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_PRIZE_CARD",
        PromptKind::ChoosePrize { count: take as u8, blocked: SVec::new(), use_opponent_prizes: false, allow_cancel: false, is_secret: false, destination: None, face_down_only: false },
        Cont::TakePrizes { p: p as u8, destination: ListRef::Hand(p as u8) },
    );
    Ok(())
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
        // No Pokémon in play (an Active spot waiting for a promotion from the Bench is not a loss).
        if pl.slots[pl.active as usize].cards.is_empty() && !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            points[1 - i] += 1;
        }
        if pl.prizes[..pl.prize_count as usize].iter().all(|l| l.is_empty()) {
            points[i] += 1;
        }
    }
    // Both players met a win condition at the same time: the one who met more wins; the same
    // number of them means Sudden Death (R7F-12; rulings 234, 820, 1403).
    if points[0] > 0 && points[1] > 0 && points[0] == points[1] {
        return initiate_sudden_death(g);
    }
    if points[0] + points[1] == 0 {
        // A Tiebreaker game is over as soon as a player has Prize advantage: fewer Prize cards remaining than
        // the opponent, after everything has resolved (rulings 567, 580).
        if g.st.is_sudden_death {
            let (a, b) = (g.st.players[0].prize_left(), g.st.players[1].prize_left());
            if a != b {
                end_game(g, if a < b { WINNER_P1 } else { WINNER_P2 });
            }
        }
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
        // Remove empty slots, starting from the right side.
        let pl = &g.st.players[p];
        let len = pl.bench.len();
        let mut empty: SVec<SlotId, 8> = SVec::new();
        for i in (0..len).rev() {
            let s = pl.bench.as_slice()[i];
            if len - empty.len() > size && pl.slots[s as usize].cards.is_empty() {
                empty.push(s);
            }
        }
        if len - empty.len() <= size {
            let pl = &mut g.st.players[p];
            for i in (0..pl.bench.len()).rev() {
                let s = pl.bench.as_slice()[i];
                if empty.contains(&s) {
                    pl.bench.remove_at(i);
                    pl.free_slot(s);
                }
            }
            continue;
        }
        // More Pokémon than the new size: the player discards some.
        let count = (len - empty.len() - size) as u8;
        let mut mask = 0u16;
        for s in empty.iter() {
            mask |= 1 << *s;
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DISCARD",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: count, max: count, allow_cancel: false, blocked: SVec::new() },
            Cont::BenchShrink { p: p as u8, empty: mask },
        );
    }
    g.st.bench_size_change_handled = true;
}

/// handleBenchSizeChange prompt callback: discard the chosen Benched Pokémon
/// and drop them and the empty slots from the Bench.
pub fn bench_shrink_cont(g: &mut Game, p: u8, empty: u16, res: Res) -> R {
    let pu = p as usize;
    let chosen: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }> = match res {
        Res::Slots(s) => s,
        _ => SVec::new(),
    };
    let discard = ListRef::Discard(p);
    let mut i = g.st.players[pu].bench.len();
    while i > 0 {
        i -= 1;
        let s = g.st.players[pu].bench.as_slice()[i];
        let selected = empty & (1 << s) != 0 || chosen.iter().any(|t| t.p == p && t.s == s);
        if !selected {
            continue;
        }
        let pokemons = g.st.slot_pokemons(pu, s);
        let slot = *g.st.slot(pu, s);
        let others: Vec<CardId> =
            slot.cards.iter().filter(|c| !g.st.cdef(*c).is_pokemon() && !pokemons.contains(c) && !slot.tools.contains(*c)).collect();
        if !others.is_empty() {
            crate::prefabs::move_cards(g, ListRef::Slot(p, s), discard, &others, NO_CARD)?;
        }
        let tools: Vec<CardId> = g.st.slot(pu, s).tools.iter().collect();
        for t in tools {
            g.move_card_to(ListRef::Slot(p, s), t, discard);
        }
        if !pokemons.is_empty() {
            crate::prefabs::move_cards(g, ListRef::Slot(p, s), discard, pokemons.as_slice(), NO_CARD)?;
        }
        let pl = &mut g.st.players[pu];
        if let Some(j) = pl.bench.position(&s) {
            pl.bench.remove_at(j);
        }
        pl.free_slot(s);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// checkStateReducer

pub fn check_state_reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::CheckAttackCost { p, .. } => {
            // attackCostIncreaseWhileActive / ignoreAttackCostCardTypes: not modeled.
            // attackCostIncreaseNextTurn: one more [C] per point (Rillaboom's Drum Beating).
            let a = g.st.players[p as usize].active;
            let n = g.st.slot(p as usize, a).attack_cost_increase_next_turn;
            if let Effect::CheckAttackCost { cost, reduction, .. } = g.e_mut(id) {
                for _ in 0..n.max(0) {
                    cost.push(ct::COLORLESS);
                }
                // "[C] less" effects (Counter Gain, Hop's Choice Band, Incineroar ex, Crabominable, Bloodmoon Ursaluna ex)
                // are applied once, together with the increases, whatever the handler order (Advanced Rulebook D-11, D-12).
                for _ in 0..*reduction {
                    match cost.iter().position(|c| *c == ct::COLORLESS) {
                        Some(i) => {
                            cost.remove_at(i);
                        }
                        None => break,
                    }
                }
            }
            // "Costs 1 Energy less" of any type (Sparkling Crystal): the Energy attached to the Pokemon covers each
            // cost slot, one slot may stay open.
            let (any_reduction, cost_now) = match *g.e(id) {
                Effect::CheckAttackCost { any_reduction, cost, .. } => (any_reduction, cost),
                _ => unreachable!(),
            };
            if any_reduction && cost_now.len() > 0 {
                let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: SlotRef::new(p as usize, a), energy_map: SVec::new() })?;
                let mut available: Vec<CardType> = Vec::new();
                if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                    for en in energy_map.iter() {
                        for t in en.provides.iter() {
                            available.push(*t);
                        }
                    }
                }
                let mut contained: Vec<CardType> = Vec::new();
                for ct_ in cost_now.iter() {
                    if *ct_ == ct::COLORLESS && !available.is_empty() {
                        contained.push(available.remove(0));
                        continue;
                    }
                    if let Some(i) = available.iter().position(|x| x == ct_) {
                        contained.push(available.remove(i));
                        continue;
                    }
                    if let Some(i) = available.iter().position(|x| *x == ct::ANY) {
                        contained.push(available.remove(i));
                    }
                }
                if contained.len() + 1 >= cost_now.len() {
                    let mut out: Cost = SVec::new();
                    for x in contained {
                        out.push(x);
                    }
                    if let Effect::CheckAttackCost { cost, .. } = g.e_mut(id) {
                        *cost = out;
                    }
                }
            }
            if let Effect::CheckAttackCost { cost, set_cost, ignore_colorless, .. } = g.e_mut(id) {
                // A cost that an effect set or ignored is final (R7F-11; rulings 147,
                // 252, 1552, 1581, 1842): CheckAttackCostEffect.setCost / ignoreColorless.
                if let Some(c) = *set_cost {
                    *cost = c;
                } else if *ignore_colorless {
                    let mut v: Cost = SVec::new();
                    for t in cost.iter() {
                        if *t != ct::COLORLESS {
                            v.push(*t);
                        }
                    }
                    *cost = v;
                }
            }
            Ok(())
        }
        Effect::CheckRetreatCost { no_cost, reduction, .. } => {
            // zeroRetreatCostNextTurn: not modeled. A "no Retreat Cost" effect
            // (noRetreatCost) takes priority over increases, whatever the
            // handler order (phase 4b, R2). retreatCostIncreaseNextTurn
            // (Rillaboom's Drum Beating) is part of the base cost:
            // retreat::check_retreat_cost_base.
            if no_cost {
                if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(id) {
                    cost.clear();
                }
            } else if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(id) {
                // "Retreat Cost is [C] less" effects (Air Balloon, Rescue Board) add up and are applied once, together
                // with the increases, whatever the handler order (Advanced Rulebook D-11, D-12).
                for _ in 0..reduction {
                    match cost.iter().position(|c| *c == ct::COLORLESS) {
                        Some(i) => {
                            let mut out: Cost = SVec::new();
                            for (j, c) in cost.iter().enumerate() {
                                if j != i {
                                    out.push(*c);
                                }
                            }
                            *cost = out;
                        }
                        None => break,
                    }
                }
            }
            Ok(())
        }
        Effect::CheckProvidedEnergy { source, .. } => {
            let slot = *g.st.slot(source.p as usize, source.s);
            let mut add: SVec<EnergyEntry, 64> = SVec::new();
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
                // `energies` entries missing from the map (Pokémon-as-energy, or
                // energy attached without leaving its list, e.g. Metang's
                // Metal Maker): `(c as any).provides || []`, skipped if empty.
                for c in slot.energies.iter() {
                    let d = g.st.cdef(c);
                    if !d.provides.is_empty() && !energy_map.iter().any(|e| e.card == c) && !add.iter().any(|e| e.card == c) {
                        let mut provides = SVec::new();
                        for &t in d.provides {
                            provides.push(t);
                        }
                        add.push(EnergyEntry { card: c, provides });
                    }
                }
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

/// `initiateSuddenDeath`: a Tiebreaker game is a new game (Advanced Player's Rulebook I-E; rulings 234, 820, 1403,
/// 1487). Every zone, the tools and the Supporter back to the deck (PokemonCardList.moveTo semantics, so attached
/// energies are pushed twice), a fresh Player for everything else (Pokémon slots, Prize lists, markers, per-turn and
/// per-game flags), the card instance flags and the last attacks reset, shuffle, then flip for the first player.
fn initiate_sudden_death(g: &mut Game) -> R {
    for p in 0..2u8 {
        let pl = &g.st.players[p as usize];
        let mut slot_lists: Vec<ListRef> = vec![ListRef::Slot(p, pl.active)];
        for b in pl.bench.iter() {
            slot_lists.push(ListRef::Slot(p, *b));
        }
        for l in slot_lists.iter() {
            let (sp, ss) = match *l {
                ListRef::Slot(a, b) => (a as usize, b),
                _ => continue,
            };
            let tools: Vec<CardId> = g.st.slot(sp, ss).tools.iter().collect();
            for t in tools {
                g.move_card_to(*l, t, ListRef::Deck(p));
            }
        }
        let pl = &g.st.players[p as usize];
        let mut lists: Vec<ListRef> = slot_lists;
        lists.push(ListRef::Discard(p));
        for i in 0..pl.prize_count {
            lists.push(ListRef::Prize(p, i));
        }
        lists.push(ListRef::Hand(p));
        lists.push(ListRef::LostZone(p));
        lists.push(ListRef::Stadium(p));
        lists.push(ListRef::Supporter(p));
        for l in lists {
            g.move_to(l, ListRef::Deck(p), None);
        }
        let old = &g.st.players[p as usize];
        let (id, deck) = (old.id, old.deck);
        let cards: Vec<CardId> = deck.iter().collect();
        g.st.players[p as usize] = Player::new(id);
        g.st.players[p as usize].deck = deck;
        for c in cards {
            let inst = &mut g.st.cards[c as usize];
            inst.moved_to_active_this_turn = false;
            inst.extra_prizes = false;
            inst.strafe_used = false;
            inst.discarded_stadium_card = false;
            inst.damage_taken_last_turn = 0;
        }
        g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p });
    }
    g.st.last_attack = None;
    g.st.player_last_attack = [None, None];
    g.st.player_last_attack_turn = [0, 0];
    let id = g.player_id(0);
    g.prompt(id, "SETUP_WHO_BEGINS_FLIP", PromptKind::CoinFlip, Cont::SuddenDeathCoin);
    Ok(())
}

/// `setupSuddenDeathGame`.
pub fn setup_sudden_death_game(g: &mut Game, first: u8) -> R {
    g.st.active_player = first;
    g.st.turn = 0;
    g.st.phase = GamePhase::Setup;
    g.st.is_sudden_death = true;
    crate::engine::setup::start(g)
}
