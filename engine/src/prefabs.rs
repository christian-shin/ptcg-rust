//! Ports of Twinleaf's shared helpers (`game/store/prefabs`), with the same
//! names in snake case. Helpers that open prompts continue through
//! [`PrefabCont`], so card ports can reuse them without their own frames.

use crate::effects::*;
use crate::game::{CoinCb, Cont, Game, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, Debug)]
pub enum PrefabCont {
    /// SHUFFLE_HAND_INTO_DECK_THEN_DRAW: hand→deck animation wait done.
    /// `after`: the `afterDraw` callback, a card continuation resumed with no results.
    ShuffleThenDraw { p: u8, draw: u8, after: Option<(CardId, crate::cards::CardFrame)> },
    /// Shuffle order chosen: apply, then wait, then draw.
    ShuffleOrderThenDraw { p: u8, draw: u8, after: Option<(CardId, crate::cards::CardFrame)> },
    DrawAfterWait { p: u8, draw: u8, after: Option<(CardId, crate::cards::CardFrame)> },
    /// SWITCH_IN_OPPONENT_BENCHED_POKEMON callback.
    SwitchInOpponent { p: u8 },
    /// THIS_ATTACK_DOES_X_DAMAGE_TO_1_OF_YOUR_OPPONENTS_[BENCHED_]POKEMON.
    DamageChosen { atk: EffId, damage: i32 },
    /// SEARCH_DECK_FOR_CARDS_TO_HAND.
    SearchToHand { p: u8, source: CardId, show: bool },
    /// SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_INTO_HAND.
    SearchPokemonToHand { p: u8 },
    /// SWITCH_ACTIVE_WITH_BENCHED callback.
    SwitchActiveWithBenched { p: u8 },
    /// SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_ONTO_BENCH: empty slots at prompt time.
    SearchToBench { p: u8, slots: SVec<SlotId, 8> },
}

pub fn resume(g: &mut Game, c: PrefabCont, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    match c {
        PrefabCont::ShuffleThenDraw { p, draw, after } => {
            shuffle_then_draw(g, p as usize, draw, after);
            Ok(())
        }
        PrefabCont::ShuffleOrderThenDraw { p, draw, after } => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[p as usize].deck, o.as_slice());
            }
            let id = g.player_id(p as usize);
            g.wait(id, Cont::Prefab(PrefabCont::DrawAfterWait { p, draw, after }));
            Ok(())
        }
        PrefabCont::DrawAfterWait { p, draw, after } => {
            draw_cards(g, p as usize, draw as usize)?;
            if let Some((card, frame)) = after {
                crate::cards::resume(g, card, frame, &[])?;
            }
            Ok(())
        }
        PrefabCont::SearchPokemonToHand { p } => {
            let cards: Vec<CardId> = first.cards().to_vec();
            show_cards_to_player(g, 1 - p as usize, cards.len());
            for c in cards {
                // MOVE_CARD_TO: findCardList(card).moveCardTo(card, hand).
                if let Some(src) = g.st.locate(c) {
                    g.move_card_to(src, c, ListRef::Hand(p));
                }
            }
            shuffle_deck(g, p as usize);
            Ok(())
        }
        PrefabCont::DamageChosen { atk, damage } => {
            let sel = match first {
                Res::Slots(s) => s,
                // `selected[0]` on a cancelled prompt throws in Twinleaf.
                _ => crate::bail!("TypeError: Cannot read properties of null"),
            };
            let t = match sel.get(0) {
                Some(t) => *t,
                None => crate::bail!("TypeError: Cannot read properties of undefined"),
            };
            let r = deal_or_put_damage(g, atk, damage, t);
            g.release_fx(atk);
            r
        }
        PrefabCont::SearchToHand { p, source, show } => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if show {
                show_cards_to_player(g, 1 - p as usize, cards.len());
            }
            move_cards(g, ListRef::Deck(p), ListRef::Hand(p), &cards, source)?;
            shuffle_deck(g, p as usize);
            Ok(())
        }
        PrefabCont::SearchToBench { p, slots } => {
            let cards: Vec<CardId> = first.cards().to_vec();
            for (i, c) in cards.iter().enumerate() {
                let s = match slots.get(i) {
                    Some(s) => *s,
                    // `slots[index]` undefined: the reducer then throws on `.cards`.
                    None => crate::bail!("TypeError: Cannot read properties of undefined"),
                };
                g.run_fx(Effect::PlayPokemonFromDeck { p, card: *c, target: SlotRef::new(p as usize, s) })?;
            }
            shuffle_deck(g, p as usize);
            Ok(())
        }
        PrefabCont::SwitchActiveWithBenched { p } => {
            let sel = first.slots();
            if sel.is_empty() {
                return Ok(());
            }
            if sel[0].p == p {
                crate::engine::turn::switch_pokemon(g, p as usize, sel[0].s)?;
            }
            Ok(())
        }
        PrefabCont::SwitchInOpponent { p } => {
            let sel = first.slots();
            if sel.is_empty() {
                return Ok(());
            }
            let o = 1 - p as usize;
            // switchPokemon only acts when the slot is on the opponent's bench.
            if sel[0].p as usize == o {
                crate::engine::turn::switch_pokemon(g, o, sel[0].s)?;
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Effect predicates

/// `WAS_ATTACK_USED(effect, index, this)`.
pub fn was_attack_used(g: &Game, e: EffId, index: u8, me: CardId) -> bool {
    matches!(*g.e(e), Effect::Attack { attack, .. } if attack == AttackRef { card: me, index })
}

/// `WAS_POWER_USED(effect, index, this)` (the lock probe never matches).
pub fn was_power_used(g: &Game, e: EffId, index: u8, me: CardId) -> bool {
    matches!(*g.e(e), Effect::Power { power, probe: false, .. } if power == PowerRef { card: me, index })
}

/// `effect instanceof TrainerEffect && effect.trainerCard === this`: the player.
pub fn trainer_played(g: &Game, e: EffId, me: CardId) -> Option<usize> {
    match *g.e(e) {
        Effect::Trainer { p, card, .. } if card == me => Some(p as usize),
        _ => None,
    }
}

/// `player.forEachPokemon(playerType, (cardList, pokemonCard, target) => ...)`:
/// slots with a Pokémon card, Active first, with their targets.
pub fn for_each_pokemon(g: &Game, p: usize, player_type: PlayerType) -> SVec<(SlotId, CardId, CardTarget), 9> {
    let mut out = SVec::new();
    let pl = &g.st.players[p];
    if let Some(c) = g.st.slot_pokemon(p, pl.active) {
        out.push((pl.active, c, CardTarget::new(player_type, SlotType::Active, 0)));
    }
    for (i, &b) in pl.bench.iter().enumerate() {
        if let Some(c) = g.st.slot_pokemon(p, b) {
            out.push((b, c, CardTarget::new(player_type, SlotType::Bench, i as u8)));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Card movement

/// `MOVE_CARDS(store, state, source, destination, { cards })`.
pub fn move_cards(g: &mut Game, src: ListRef, dst: ListRef, cards: &[CardId], source_card: CardId) -> R {
    let cs: List<120> = List::from_slice(cards);
    g.run_fx(Effect::MoveCards {
        source: src,
        destination: dst,
        cards: Some(cs),
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card,
    })?;
    Ok(())
}

/// `MOVE_CARDS(..., { count })`.
pub fn move_count(g: &mut Game, src: ListRef, dst: ListRef, count: usize) -> R {
    move_count_from(g, src, dst, count, NO_CARD)
}

/// `MOVE_CARDS(..., { count, sourceCard })`.
pub fn move_count_from(g: &mut Game, src: ListRef, dst: ListRef, count: usize, source_card: CardId) -> R {
    g.run_fx(Effect::MoveCards {
        source: src,
        destination: dst,
        cards: None,
        count: Some(count as i32),
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card,
    })?;
    Ok(())
}

/// `DRAW_CARDS`.
pub fn draw_cards(g: &mut Game, p: usize, count: usize) -> R {
    let n = count.min(g.st.players[p].deck.len());
    if n == 0 {
        return Ok(());
    }
    move_count(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), n)
}

/// `SHUFFLE_DECK`: shuffle prompt, then a silent wait.
pub fn shuffle_deck(g: &mut Game, p: usize) {
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApply { p: p as u8 });
}

fn shuffle_then_draw(g: &mut Game, p: usize, draw: u8, after: Option<(CardId, crate::cards::CardFrame)>) {
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Prefab(PrefabCont::ShuffleOrderThenDraw { p: p as u8, draw, after }));
}

/// `SHUFFLE_HAND_INTO_DECK_THEN_DRAW(store, state, player, { excludeCard, drawCount })`.
pub fn shuffle_hand_into_deck_then_draw(g: &mut Game, p: usize, exclude: CardId, draw: u8) -> R {
    shuffle_hand_into_deck_then_draw_ex(g, p, exclude, NO_CARD, draw, None)
}

/// `SHUFFLE_HAND_INTO_DECK_THEN_DRAW` with `sourceCard` and an `afterDraw`
/// callback (a card continuation resumed with no results after the draw).
pub fn shuffle_hand_into_deck_then_draw_ex(
    g: &mut Game,
    p: usize,
    exclude: CardId,
    source_card: CardId,
    draw: u8,
    after: Option<(CardId, crate::cards::CardFrame)>,
) -> R {
    let cards: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != exclude).collect();
    if !cards.is_empty() {
        let cs: List<120> = List::from_slice(&cards);
        let (_, prevented) = g.run_fx(Effect::MoveCards {
            source: ListRef::Hand(p as u8),
            destination: ListRef::Deck(p as u8),
            cards: Some(cs),
            count: None,
            to_top: false,
            to_bottom: false,
            skip_cleanup: false,
            source_card,
        })?;
        if prevented {
            return Ok(());
        }
        let id = g.player_id(p);
        g.wait(id, Cont::Prefab(PrefabCont::ShuffleThenDraw { p: p as u8, draw, after }));
        return Ok(());
    }
    shuffle_then_draw(g, p, draw, after);
    Ok(())
}

/// `SWITCH_IN_OPPONENT_BENCHED_POKEMON(store, state, player, { allowCancel })`.
pub fn switch_in_opponent_benched_pokemon(g: &mut Game, p: usize, allow_cancel: bool) {
    let o = 1 - p;
    let pl = &g.st.players[o];
    if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
        return;
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_SWITCH",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel, blocked: SVec::new() },
        Cont::Prefab(PrefabCont::SwitchInOpponent { p: p as u8 }),
    );
}

/// `SWITCH_ACTIVE_WITH_BENCHED(store, state, player)`.
pub fn switch_active_with_benched(g: &mut Game, p: usize) {
    let pl = &g.st.players[p];
    if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
        return;
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_NEW_ACTIVE_POKEMON",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Prefab(PrefabCont::SwitchActiveWithBenched { p: p as u8 }),
    );
}

/// `MOVE_POKEMON_OFF_BOARD(store, state, slot, { pokemonDestination, sourceCard })`
/// with no separate `attachedDestination`: one full-stack MOVE_CARDS.
pub fn move_pokemon_off_board(g: &mut Game, slot: SlotRef, destination: ListRef, source_card: CardId) -> R {
    g.run_fx(Effect::MoveCards {
        source: slot.list(),
        destination,
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card,
    })?;
    Ok(())
}

/// `CONFIRMATION_PROMPT(store, state, player, callback, message)`.
pub fn confirmation_prompt(g: &mut Game, p: usize, message: &'static str, cont: Cont) {
    let id = g.player_id(p);
    g.prompt(id, message, PromptKind::Confirm, cont);
}

/// `OPPONENT_CANNOT_PLAY_CARDS(store, state, effect, source, options)`:
/// reduce a `PlayLockEffect` (default durations).
pub fn opponent_cannot_play_cards(g: &mut Game, atk: EffId, locks: u16) -> R {
    let mut b = match *g.e(atk) {
        Effect::Attack { p, opp, attack, source, .. } => AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: source },
        _ => return Ok(()),
    };
    b.target = b.source;
    g.run_fx(Effect::PlayLock { b, locks, turns_remaining: None, both_players: false, attacker_turns_remaining: None })?;
    Ok(())
}

/// `new ChooseCardsPrompt(player, message, list, filter, options)`, including
/// the constructor's sort of a non-secret deck or discard.
pub fn choose_cards(g: &mut Game, p: usize, message: &'static str, list: ListRef, filter: Filter, mut opts: ChooseCardsOpts, cont: Cont) {
    if !opts.is_secret && (list == ListRef::Deck(p as u8) || list == ListRef::Discard(p as u8)) {
        // Blocked indices follow the cards through the sort.
        let before: Vec<CardId> = g.lst(list).to_vec();
        let blocked_cards: Vec<CardId> = before.iter().enumerate().filter(|(i, _)| opts.blocked.contains(&(*i as u8))).map(|(_, c)| *c).collect();
        g.sort_list(list);
        if !blocked_cards.is_empty() {
            let mut b = Blocked::default();
            for (i, c) in g.lst(list).iter().enumerate() {
                if blocked_cards.contains(c) {
                    b.push(i as u8);
                }
            }
            opts.blocked = b;
        }
    }
    let id = g.player_id(p);
    g.prompt(id, message, PromptKind::ChooseCards { cards: list, filter, opts }, cont);
}

// ---------------------------------------------------------------------------
// Probes

/// Power flags carried by a lock probe (`IS_ABILITY_BLOCKED`'s stand-in power).
/// `index == PROBE_GENERIC` means a plain Ability with no flags.
pub const PROBE_GENERIC: u8 = 255;

/// `IS_ABILITY_BLOCKED(store, state, player, card[, power])`.
pub fn is_ability_blocked(g: &mut Game, p: usize, card: CardId, power_index: Option<u8>) -> bool {
    let power = PowerRef { card, index: power_index.unwrap_or(PROBE_GENERIC) };
    g.run_fx(Effect::Power { p: p as u8, power, card, target: None, probe: true }).is_err()
}

/// The power type of a (possibly probe) `PowerEffect`.
pub fn power_type_of(g: &Game, power: PowerRef) -> u8 {
    if power.index == PROBE_GENERIC {
        PowerType::Ability as u8
    } else {
        g.st.cdef(power.card).powers[power.index as usize].power_type
    }
}

/// `IS_TOOL_BLOCKED`.
pub fn is_tool_blocked(g: &mut Game, p: usize, card: CardId) -> bool {
    if g.st.players.iter().any(|pl| pl.stadium_and_tool_have_no_effect_turns_remaining > 0) {
        return true;
    }
    g.run_fx(Effect::Tool { p: p as u8, card }).is_err()
}

/// `IS_STADIUM_EFFECT_BLOCKED`.
pub fn is_stadium_effect_blocked(g: &mut Game, p: usize, target: SlotRef, stadium: CardId) -> bool {
    if g.st.players.iter().any(|pl| pl.stadium_and_tool_have_no_effect_turns_remaining > 0) {
        return true;
    }
    if g.probing_stadium {
        return g.run_fx(Effect::Stadium { p: p as u8, target: Some(target), stadium, skip_ability_lock_check: true }).is_err();
    }
    g.probing_stadium = true;
    let r = g.run_fx(Effect::Stadium { p: p as u8, target: Some(target), stadium, skip_ability_lock_check: false });
    g.probing_stadium = false;
    r.is_err()
}

/// `IS_SPECIAL_ENERGY_BLOCKED`.
pub fn is_special_energy_blocked(g: &mut Game, p: usize, card: CardId, attached_to: SlotRef, exempt: bool) -> bool {
    g.run_fx(Effect::SpecialEnergy { p: p as u8, card, attached_to, exempt }).is_err()
}

// ---------------------------------------------------------------------------
// Markers and ability bookkeeping

/// `ABILITY_USED(player, card)`: board effect on the slot holding `card`.
pub fn ability_used(g: &mut Game, p: usize, card: CardId) {
    for s in g.st.players[p].in_play().iter() {
        if g.st.slot_pokemon(p, *s) == Some(card) {
            let be = &mut g.st.players[p].slots[*s as usize].board_effect;
            let v = BoardEffect::AbilityUsed as u8;
            if !be.contains(&v) {
                be.retain(|b| !matches!(*b, 0..=3));
                be.push(v);
            }
        }
    }
}

/// `USE_ABILITY_ONCE_PER_TURN(player, marker, source)` on the player marker.
pub fn use_ability_once_per_turn(g: &mut Game, p: usize, marker: crate::markers::MarkerName, source: CardId) -> R {
    if g.st.players[p].marker.has_from(marker, source) {
        crate::bail!("POWER_ALREADY_USED");
    }
    g.st.players[p].marker.add(marker, source, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    Ok(())
}

/// `REMOVE_MARKER_AT_END_OF_TURN(effect, marker, source)` (player marker).
pub fn remove_marker_at_end_of_turn(g: &mut Game, e: EffId, marker: crate::markers::MarkerName, source: CardId) {
    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(marker, source) {
            m.remove_from(marker, source);
        }
    }
}

// ---------------------------------------------------------------------------
// Attack helpers

fn atk_base_for(g: &Game, atk: EffId, target: SlotRef) -> AtkBase {
    match *g.e(atk) {
        Effect::Attack { p, opp, attack, source, .. } => AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target },
        _ => panic!("not an attack effect"),
    }
}

/// DealDamageEffect on the opponent's Active, PutDamageEffect elsewhere.
pub fn deal_or_put_damage(g: &mut Game, atk: EffId, damage: i32, target: SlotRef) -> R {
    let b = atk_base_for(g, atk, target);
    let o = b.opponent as usize;
    if target.p as usize == o && target.s == g.st.players[o].active {
        g.run_fx(Effect::DealDamage { b, damage })?;
    } else {
        g.run_fx(Effect::PutDamage { b, damage, weakness_applied: false, survive_on_ten_hp: false })?;
    }
    Ok(())
}

/// `DAMAGE_OPPONENT_POKEMON(store, state, effect, damage, targets)`.
pub fn damage_opponent_pokemon(g: &mut Game, atk: EffId, damage: i32, targets: &[SlotRef]) -> R {
    for t in targets {
        deal_or_put_damage(g, atk, damage, *t)?;
    }
    Ok(())
}

/// `PutDamageEffect(effect, damage)` on `target` (no Weakness for the Bench).
pub fn put_damage(g: &mut Game, atk: EffId, damage: i32, target: SlotRef) -> R {
    let b = atk_base_for(g, atk, target);
    g.run_fx(Effect::PutDamage { b, damage, weakness_applied: false, survive_on_ten_hp: false })?;
    Ok(())
}

/// `THIS_ATTACK_DOES_X_DAMAGE_TO_1_OF_YOUR_OPPONENTS_BENCHED_POKEMON` (bench_only)
/// and `..._TO_1_OF_YOUR_OPPONENTS_POKEMON`.
pub fn damage_1_opponent_pokemon(g: &mut Game, atk: EffId, damage: i32, bench_only: bool) {
    let p = match *g.e(atk) {
        Effect::Attack { p, .. } => p as usize,
        _ => return,
    };
    let o = 1 - p;
    let pl = &g.st.players[o];
    let has = if bench_only {
        pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty())
    } else {
        !pl.in_play().is_empty()
    };
    if !has {
        return;
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    if !bench_only {
        slots.push(SlotType::Active as u8);
    }
    g.retain_fx(atk);
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: true, blocked: SVec::new() },
        Cont::Prefab(PrefabCont::DamageChosen { atk, damage }),
    );
}

// ---------------------------------------------------------------------------
// Deck search

/// `SHOW_CARDS_TO_PLAYER(store, state, player, cards)`: info prompt if any.
pub fn show_cards_to_player(g: &mut Game, p: usize, n_cards: usize) {
    if n_cards == 0 {
        return;
    }
    let id = g.player_id(p);
    g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
}

/// `SEARCH_DECK_FOR_CARDS_TO_HAND(store, state, player, sourceCard, filter, options)`.
pub fn search_deck_for_cards_to_hand(g: &mut Game, p: usize, source: CardId, filter: Filter, opts: ChooseCardsOpts) {
    if g.st.players[p].deck.is_empty() {
        return;
    }
    let show = filter != Filter::none();
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, opts, Cont::Prefab(PrefabCont::SearchToHand { p: p as u8, source, show }));
}

/// `SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_INTO_HAND(store, state, player, filter, options)`.
pub fn search_deck_for_pokemon_to_hand(g: &mut Game, p: usize, mut filter: Filter, opts: ChooseCardsOpts) -> R {
    if g.st.players[p].deck.is_empty() {
        crate::bail!("NO_CARDS_IN_DECK");
    }
    filter.super_type = Some(SuperType::Pokemon as u8);
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, opts, Cont::Prefab(PrefabCont::SearchPokemonToHand { p: p as u8 }));
    Ok(())
}

/// `GET_PLAYER_BENCH_SLOTS`: empty bench slots in order.
pub fn empty_bench_slots(g: &Game, p: usize) -> SVec<SlotId, 8> {
    let pl = &g.st.players[p];
    let mut v = SVec::new();
    for &b in pl.bench.iter() {
        if pl.slots[b as usize].cards.is_empty() {
            v.push(b);
        }
    }
    v
}

/// `SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_ONTO_BENCH(store, state, player, filter, options)`.
pub fn search_deck_for_pokemon_to_bench(g: &mut Game, p: usize, mut filter: Filter, opts: ChooseCardsOpts) -> R {
    if g.st.players[p].deck.is_empty() {
        crate::bail!("NO_CARDS_IN_DECK");
    }
    let slots = empty_bench_slots(g, p);
    if slots.is_empty() {
        crate::bail!("NO_BENCH_SLOTS_AVAILABLE");
    }
    filter.super_type = Some(SuperType::Pokemon as u8);
    choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Deck(p as u8), filter, opts, Cont::Prefab(PrefabCont::SearchToBench { p: p as u8, slots }));
    Ok(())
}

/// `MULTIPLE_COIN_FLIPS_PROMPT` / `FLIP_UNTIL_TAILS` (`mode` 0 = until tails):
/// the callback receives the results as a bitmask (bit i = flip i heads) and count.
pub fn coin_flip_sequence(g: &mut Game, p: usize, mode: u8, cb: CoinCb) -> R {
    g.coin_callbacks.push(cb);
    let k = (g.coin_callbacks.len() - 1) as u8;
    g.run_fx(Effect::CoinFlipSequence { p: p as u8, mode, callback: k, skip_reflip_stadium: false, skip_reflip_tool: false })?;
    Ok(())
}
