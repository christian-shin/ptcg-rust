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
    SwitchInOpponent { p: u8, cause: crate::cause::Cause },
    /// THIS_ATTACK_DOES_X_DAMAGE_TO_1_OF_YOUR_OPPONENTS_[BENCHED_]POKEMON.
    DamageChosen { atk: EffId, damage: i32 },
    /// SEARCH_DECK_FOR_CARDS_TO_HAND.
    SearchToHand { p: u8, source: CardId, show: bool },
    /// SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_INTO_HAND.
    SearchPokemonToHand { p: u8 },
    /// SWITCH_ACTIVE_WITH_BENCHED callback.
    SwitchActiveWithBenched { p: u8, cause: crate::cause::Cause },
    /// SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_ONTO_BENCH: empty slots at prompt time.
    SearchToBench { p: u8, slots: SVec<SlotId, 8>, cause: crate::cause::Cause },
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
                // Not cancellable (`allowCancel: false`), so a result is always given.
                _ => return Ok(()),
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
        PrefabCont::SearchToBench { p, slots, cause } => {
            let cards: Vec<CardId> = first.cards().to_vec();
            for (i, c) in cards.iter().enumerate() {
                let s = match slots.get(i) {
                    Some(s) => *s,
                    // The prompt's `max` is clamped to the empty slots.
                    None => break,
                };
                g.run_fx_unit(Effect::PlayPokemonFromDeck { p, card: *c, target: SlotRef::new(p as usize, s), cause })?;
            }
            shuffle_deck(g, p as usize);
            Ok(())
        }
        PrefabCont::SwitchActiveWithBenched { p, cause } => {
            let sel = first.slots();
            if sel.is_empty() {
                return Ok(());
            }
            if sel[0].p == p {
                crate::engine::turn::switch_pokemon(g, p as usize, sel[0].s, cause)?;
            }
            Ok(())
        }
        PrefabCont::SwitchInOpponent { p, cause } => {
            let sel = first.slots();
            if sel.is_empty() {
                return Ok(());
            }
            let o = 1 - p as usize;
            // switchPokemon only acts when the slot is on the opponent's bench.
            if sel[0].p as usize == o {
                crate::engine::turn::switch_pokemon(g, o, sel[0].s, cause)?;
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Effect predicates

/// `this.attacks[index]`: while a copy-attack source's code runs as
/// `.call(copycat)`, the copycat's attacks are the session's clones.
pub fn my_attack(g: &Game, me: CardId, index: u8) -> AttackRef {
    match g.deleg {
        Some(d) if d.attacks && d.copycat == me => crate::copy_attack::clone_ref(d.source, d.serial, index),
        _ => AttackRef { card: me, index },
    }
}

/// `WAS_ATTACK_USED(effect, index, this)`.
///
/// While a copy-attack session runs a source handler for the copycat, the
/// copycat's attacks are the session's clones (`withTemporaryDelegatedAttacks`).
pub fn was_attack_used(g: &Game, e: EffId, index: u8, me: CardId) -> bool {
    let mine = my_attack(g, me, index);
    matches!(*g.e(e), Effect::Attack { attack, .. } if attack == mine)
}

/// `SURVIVE_ON_TEN_ON_COIN_FLIP(store, state, effect, player, reason)`: when the PutDamageEffect would Knock Out
/// its target (existing damage plus `effect.damage >=` the CheckHpEffect HP) during an attack, the Pokémon is recorded:
/// the full damage is done, then the coin is flipped and 10 HP restored AFTER all the damage (ruling 1770,
/// `resolve_survive_coin_flips`). Outside an attack the coin is flipped right away (`CoinFlipEffect.result`, no
/// callback: a callback would run after the flip's wait prompt, i.e. after the damage was applied); heads sets
/// `surviveOnTenHPReason`.
pub fn survive_on_ten_on_coin_flip(g: &mut Game, e: EffId, player: usize) -> R {
    let (t, damage) = match *g.e(e) {
        Effect::PutDamage { b, damage, .. } => (b.target, damage),
        _ => return Ok(()),
    };
    let hp = crate::engine::check::check_hp(g, player, t.s)?;
    if g.st.slot(t.p as usize, t.s).damage + damage >= hp {
        if g.st.phase == GamePhase::Attack {
            if !g.ten_hp_coin.iter().any(|(s, _)| *s == t) {
                g.ten_hp_coin.push((t, player as u8));
            }
            return Ok(());
        }
        let (c, _) = g.run_fx(Effect::CoinFlip { p: player as u8, callback: None, result: None, skip_reflip_stadium: false, skip_reflip_tool: false })?;
        if let Effect::CoinFlip { result: Some(true), .. } = c {
            if let Effect::PutDamage { survive_on_ten_hp, .. } = g.e_mut(e) {
                *survive_on_ten_hp = true;
            }
        }
    }
    Ok(())
}

/// `RESOLVE_SURVIVE_COIN_FLIPS`: flip the coins of the Pokémon that took lethal damage from the attack once all its
/// damage is done (ruling 1770): heads, it is not Knocked Out and its remaining HP becomes 10.
pub fn resolve_survive_coin_flips(g: &mut Game) -> R {
    let list = std::mem::replace(&mut g.ten_hp_coin, SVec::new());
    for (t, owner) in list.iter().copied() {
        let (tp, ts) = (t.p as usize, t.s);
        if g.st.slot_pokemon(tp, ts).is_none() {
            continue;
        }
        let hp = crate::engine::check::check_hp(g, owner as usize, ts)?;
        if g.st.slot(tp, ts).damage < hp {
            continue;
        }
        let (c, _) = g.run_fx(Effect::CoinFlip { p: owner, callback: None, result: None, skip_reflip_stadium: false, skip_reflip_tool: false })?;
        if let Effect::CoinFlip { result: Some(true), .. } = c {
            g.st.players[tp].slots[ts as usize].damage = hp - 10;
            if !g.ten_hp.contains(&t) {
                g.ten_hp.push(t);
            }
        }
    }
    Ok(())
}

/// `AFTER_ATTACK(effect, index, this)`.
pub fn after_attack_used(g: &Game, e: EffId, index: u8, me: CardId) -> bool {
    let mine = my_attack(g, me, index);
    matches!(*g.e(e), Effect::AfterAttack { attack, .. } if attack == mine)
}

/// The attack's own AttackEffect behind an AfterAttackEffect (`effect.attackEffect`); any other effect is returned
/// as is. A block moved to AfterAttack (effect text asked after the damage) calls this first and then reads the
/// attack's state as before.
pub fn real_attack(g: &Game, e: EffId) -> EffId {
    match *g.e(e) {
        Effect::AfterAttack { atk, .. } => atk,
        _ => e,
    }
}

/// (player, opponent, attack, source) of an AttackEffect, or of the
/// `new AttackEffect(player, opponent, effect.attack)` that a card builds in an
/// AfterAttackEffect handler (its source is the player's Active).
pub fn attack_data(g: &Game, e: EffId) -> Option<(u8, u8, AttackRef, SlotRef)> {
    match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => Some((p, opp, attack, source)),
        Effect::AfterAttack { p, opp, attack, .. } => Some((p, opp, attack, SlotRef::new(p as usize, g.st.players[p as usize].active))),
        _ => None,
    }
}

/// `WAS_POWER_USED(effect, index, this)` (the lock probe never matches).
pub fn was_power_used(g: &Game, e: EffId, index: u8, me: CardId) -> bool {
    matches!(*g.e(e), Effect::Power { power, probe: false, .. } if power == PowerRef { card: me, index })
}

/// `TRANSFER_POKEMON_CARD_STATE(player, oldCard, newCard)`: a Pokémon switched with another card
/// (Transformation Tome, Ogre's Mask, Zero to Hero) is the same Pokémon (ruling 1840): the state
/// kept on the card object moves to the new card.
pub fn transfer_pokemon_card_state(g: &mut Game, p: usize, old: CardId, new: CardId) {
    g.st.cards[new as usize].damage_taken_last_turn = g.st.cards[old as usize].damage_taken_last_turn;
    g.st.cards[old as usize].damage_taken_last_turn = 0;
    g.st.cards[new as usize].moved_to_active_this_turn = g.st.cards[old as usize].moved_to_active_this_turn;
    g.st.cards[old as usize].moved_to_active_this_turn = false;
    let pl = &mut g.st.players[p];
    for id in pl.moved_to_active_this_turn.as_mut_slice().iter_mut().chain(pl.moved_from_active_to_bench_this_turn.as_mut_slice().iter_mut()) {
        if *id == old {
            *id = new;
        }
    }
}

/// `effect.usedAsAttackEffect` of a TrainerEffect: the Supporter's effect is used as the
/// effect of an attack (Mr. Mime's Look-Alike Show), so "up to" prompts may choose zero.
pub fn trainer_via_attack(g: &Game, e: EffId) -> bool {
    matches!(*g.e(e), Effect::Trainer { via_attack: true, .. })
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

/// `ENERGY_CARDS_THAT_PROVIDE_TYPE(store, state, player, cardList, cardType)` (prefabs/costs.ts): the Energy
/// cards attached to the slot that provide `ty` as the Pokémon's Energy provides it right now
/// (CheckProvidedEnergyEffect); an Energy that provides every type (Legacy Energy, Prism Energy on a Basic Pokémon)
/// is a [R] Energy, a [W] Energy, ... for every "[X] Energy" in card text (Advanced Rulebook D-08).
pub fn energy_cards_that_provide_type(g: &mut Game, p: usize, s: SlotId, ty: CardType) -> R<SVec<CardId, 64>> {
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, s), energy_map: SVec::new() })?;
    let mut cards: SVec<CardId, 64> = SVec::new();
    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
        for m in energy_map.iter() {
            if (m.provides.contains(&ty) || m.provides.contains(&ct::ANY)) && !cards.contains(&m.card) {
                cards.push(m.card);
            }
        }
    }
    Ok(cards)
}

/// `BLOCKED_NON_TYPE_ENERGY(...)`: the prompt `blockedMap` indices of the slot's Energy cards that do not provide
/// `ty`; `None` when there is none.
pub fn blocked_non_type_energy(g: &mut Game, p: usize, s: SlotId, ty: CardType) -> R<Option<Blocked>> {
    let providing = energy_cards_that_provide_type(g, p, s, ty)?;
    let mut b = Blocked::default();
    let mut any = false;
    for (i, c) in g.st.slot(p, s).cards.iter().enumerate() {
        if g.st.cdef(c).is_energy() && !providing.contains(&c) {
            b.push(i as u8);
            any = true;
        }
    }
    Ok(if any { Some(b) } else { None })
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

/// `MOVE_CARDS(store, state, source, destination, { cards, afterDamageOf: attackEffect })`: Energy removed
/// as an effect of an attack leaves after the damage (queued while the attack's window is open).
pub fn move_cards_after_damage(g: &mut Game, atk: EffId, src: ListRef, dst: ListRef, cards: &[CardId], source_card: CardId) -> R {
    if !g.after_damage_open(atk) {
        return move_cards(g, src, dst, cards, source_card);
    }
    let mut cs: SVec<CardId, 64> = SVec::new();
    for &c in cards {
        cs.push(c);
    }
    g.push_after_damage(atk, crate::game::AfterDmgStep::Move { source: src, destination: dst, source_card, cards: cs });
    Ok(())
}

/// `SHUFFLE_DECK_AFTER_DAMAGE(store, state, attackEffect, player)`.
pub fn shuffle_deck_after_damage(g: &mut Game, atk: EffId, p: usize) {
    if g.after_damage_open(atk) {
        g.push_after_damage(atk, crate::game::AfterDmgStep::Shuffle(p as u8));
    } else {
        shuffle_deck(g, p);
    }
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
pub fn opponent_cannot_play_cards(g: &mut Game, atk: EffId, lock: &'static crate::spec::passive::LockDecl) -> R {
    // An AfterAttackEffect handler passes `new AttackEffect(player, opponent, effect.attack)`
    // (Chi-Yu MEG): its source is the player's Active.
    let mut b = match attack_data(g, atk) {
        Some((p, opp, attack, source)) => AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: source, cause: crate::cause::Cause::of_attack_at(g, p, attack, source) },
        None => return Ok(()),
    };
    b.target = b.source;
    g.run_fx_unit(Effect::PlayLock { b, lock })?;
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
    // A copied attack's source code, run for the copycat, must not use the source's Abilities.
    if g.deleg.map_or(false, |d| d.attacks && d.copycat == card) {
        return true;
    }
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

/// A card's own once-per-turn use marks reset when its Pokémon changes (ruling 317: evolving,
/// devolving, leaving play, and coming into play again); benching does not.
pub fn reset_once_per_turn(g: &mut Game, card: CardId) {
    let Some(spec) = crate::cards::spec_for(g.st.cards[card as usize].def) else { return };
    for pw in spec.powers.iter() {
        if let crate::spec::Once::PerTurn(name) = pw.once {
            let m = crate::markers::intern(name);
            for pl in g.st.players.iter_mut() {
                pl.marker.remove_from(m, card);
            }
        }
    }
}

/// `reset_once_per_turn` for every card of a slot.
pub fn reset_once_per_turn_slot(g: &mut Game, t: SlotRef) {
    let cards: Vec<CardId> = g.st.slot(t.p as usize, t.s).cards.iter().collect();
    for c in cards {
        reset_once_per_turn(g, c);
    }
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
        Effect::Attack { p, opp, attack, source, .. } => AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target, cause: crate::cause::Cause::of_attack_at(g, p, attack, source) },
        _ => panic!("not an attack effect"),
    }
}

/// DealDamageEffect on the opponent's Active, PutDamageEffect elsewhere.
pub fn deal_or_put_damage(g: &mut Game, atk: EffId, damage: i32, target: SlotRef) -> R {
    let b = atk_base_for(g, atk, target);
    let o = b.opponent as usize;
    if target.p as usize == o && target.s == g.st.players[o].active {
        g.run_fx_unit(Effect::DealDamage { b, damage })?;
    } else {
        g.run_fx_unit(Effect::PutDamage { b, damage, weakness_applied: false, survive_on_ten_hp: false })?;
    }
    Ok(())
}

/// `PutDamageEffect(effect, damage)` on `target` (no Weakness for the Bench).
pub fn put_damage(g: &mut Game, atk: EffId, damage: i32, target: SlotRef) -> R {
    let b = atk_base_for(g, atk, target);
    g.run_fx_unit(Effect::PutDamage { b, damage, weakness_applied: false, survive_on_ten_hp: false })?;
    Ok(())
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

/// `MULTIPLE_COIN_FLIPS_PROMPT` / `FLIP_UNTIL_TAILS` (`mode` 0 = until tails):
/// the callback receives the results as a bitmask (bit i = flip i heads) and count.
pub fn coin_flip_sequence(g: &mut Game, p: usize, mode: u8, cb: CoinCb) -> R {
    let cb = g.tag_coin(cb);
    g.coin_callbacks.push(cb);
    let k = (g.coin_callbacks.len() - 1) as u8;
    g.run_fx_unit(Effect::CoinFlipSequence { p: p as u8, mode, callback: k, skip_reflip_stadium: false, skip_reflip_tool: false })?;
    Ok(())
}

/// `ignoresDefenderEffects(effect)`: Shred ("isn't affected by any effects on your opponent's Active
/// Pokémon"). True for damage by an attack whose `AttackEffect.ignoreDefenderEffects` is set, done to one
/// of the opponent's Pokémon. Effects on the damaged Pokémon (prevention, reduction, extra damage taken,
/// Tera/Bench protection) must not change it (rulings 1439, 1345, 1490, 1629, 1875); effects on the
/// attacker, Weakness/Resistance and survive-on-10-HP effects (rulings 936, 1770) still apply.
pub fn ignores_defender_effects(g: &Game, b: &AtkBase) -> bool {
    matches!(*g.e(b.attack_effect), Effect::Attack { ignore_defender_effects: true, .. }) && b.target.p == b.opponent
}

/// `TERA_RULE(effect, state, source)`: prevent attack damage put on this
/// Pokémon while it is on the Bench.
pub fn tera_rule(g: &mut Game, e: EffId, me: CardId) {
    if let Effect::PutDamage { b, .. } = *g.e(e) {
        if ignores_defender_effects(g, &b) {
            return;
        }
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).cards.contains(me) && g.st.slot_pokemon(t.p as usize, t.s) == Some(me) {
            let pl = b.player as usize;
            let op = 1 - pl;
            if (t.p as usize == pl && t.s == g.st.players[pl].active) || (t.p as usize == op && t.s == g.st.players[op].active) {
                return;
            }
            g.set_prevent(e, true);
        }
    }
}

/// `BLOCK_RETREAT(store, state, effect, source)`: reduce a `PreventRetreatEffect`.
pub fn block_retreat(g: &mut Game, atk: EffId) -> R {
    let o = match *g.e(atk) {
        Effect::Attack { opp, .. } => opp as usize,
        _ => return Ok(()),
    };
    let target = SlotRef::new(o, g.st.players[o].active);
    let b = atk_base_for(g, atk, target);
    g.run_fx_unit(Effect::PreventRetreat { b })?;
    Ok(())
}

/// `PREVENT_DAMAGE(store, state, effect, source)` (no options): reduce a
/// `PreventDamageEffect` whose target is the attack's source.
pub fn prevent_damage(g: &mut Game, atk: EffId) -> R {
    let source = match *g.e(atk) {
        Effect::Attack { source, .. } => source,
        _ => return Ok(()),
    };
    let b = atk_base_for(g, atk, source);
    g.run_fx_unit(Effect::PreventDamage { b })?;
    Ok(())
}

/// `PREVENT_DAMAGE(store, state, effect, source, options)` with non-empty options.
pub fn prevent_damage_filtered(g: &mut Game, atk: EffId, filter: crate::state::PreventFilter) -> R {
    let source = match *g.e(atk) {
        Effect::Attack { source, .. } => source,
        _ => return Ok(()),
    };
    let b = atk_base_for(g, atk, source);
    g.run_fx_unit(Effect::PreventDamageFiltered { b, filter })?;
    Ok(())
}

/// `PREVENT_EFFECTS_OF_ATTACKS(store, state, effect, source)` (no options):
/// reduce a `PreventEffectsOfAttacksEffect` whose target is the attack's source.
pub fn prevent_effects_of_attacks(g: &mut Game, atk: EffId) -> R {
    let source = match *g.e(atk) {
        Effect::Attack { source, .. } => source,
        _ => return Ok(()),
    };
    let b = atk_base_for(g, atk, source);
    g.run_fx_unit(Effect::PreventEffectsOfAttacks { b })?;
    Ok(())
}

/// `BLOCK_SELF_RETREAT(store, state, effect, source)`: a
/// `SelfPreventRetreatEffect` whose target is the attacking Pokémon (phase 4b:
/// it used to be left at the opponent's Active, where Mist Energy etc.
/// prevented it).
pub fn block_self_retreat(g: &mut Game, atk: EffId) -> R {
    let source = match *g.e(atk) {
        Effect::Attack { source, .. } => source,
        _ => return Ok(()),
    };
    let b = atk_base_for(g, atk, source);
    g.run_fx_unit(Effect::SelfPreventRetreat { b })?;
    Ok(())
}

/// `DISCARD_ATTACKER_ENERGY_IF_THIS_POKEMON_KNOCKED_OUT_DURING_OPPONENTS_NEXT_TURN`.
pub fn discard_attacker_energy_if_knocked_out(g: &mut Game, atk: EffId, source_card: CardId) -> R {
    let source = match *g.e(atk) {
        Effect::Attack { source, .. } => source,
        _ => return Ok(()),
    };
    let b = atk_base_for(g, atk, source);
    g.run_fx_unit(Effect::DiscardAttackerEnergyIfKnockedOut { b, source_card })?;
    Ok(())
}

/// `ADD_SPECIAL_CONDITIONS_TO_PLAYER_ACTIVE(store, state, player, source, conditions)`
/// with the default poison/burn/sleep/confusion values: reduce an
/// `AddSpecialConditionsPowerEffect` on `player.active`.
pub fn add_special_conditions_to_player_active(g: &mut Game, p: usize, source: CardId, conditions: &[SpecialCondition], cause: crate::cause::Cause) -> R {
    let target = SlotRef::new(p, g.st.players[p].active);
    let mut cs = SVec::new();
    for c in conditions {
        cs.push(*c as u8);
    }
    g.run_fx_unit(Effect::AddSpecialConditionsPower { p: p as u8, source, target, conditions: cs, poison_damage: 10, burn_damage: 20, sleep_flips: 1, confusion_damage: 30, cause })?;
    Ok(())
}

/// `OPPONENT_POKEMON_WITH_X_OR_LESS_ENERGY_CANNOT_ATTACK(store, state, effect, source, maxEnergy)`:
/// an `OpponentPokemonCannotAttackDuringTheirNextTurnEffect` (target = the attacker's slot).
pub fn opponent_pokemon_with_x_or_less_energy_cannot_attack(g: &mut Game, atk: EffId, max_energy: i32) -> R {
    let source = match *g.e(atk) {
        Effect::Attack { source, .. } => source,
        _ => return Ok(()),
    };
    let b = atk_base_for(g, atk, source);
    g.run_fx_unit(Effect::OpponentPokemonCannotAttackNextTurn { b, max_energy: Some(max_energy) })?;
    Ok(())
}

/// `(card << 4) | index` of an attack, for a card frame slot.
pub fn pack_attack(a: AttackRef) -> i32 {
    ((a.card as i32) << 4) | (a.index as i32 & 15)
}

/// Does something prevent the effect of the attack `attack` (used by player `p`
/// against `o`) on the Pokémon in `target`? A DiscardCardsEffect without cards
/// on a fresh AttackEffect asks (Mist Energy and the like; R7F-17, ruling 1843).
pub fn attack_effect_prevented_on(g: &mut Game, p: usize, o: usize, packed_attack: i32, target: SlotRef, cause: crate::cause::Cause) -> R<bool> {
    let attack = AttackRef { card: (packed_attack >> 4) as CardId, index: (packed_attack & 15) as u8 };
    let source = SlotRef::new(p, g.st.players[p].active);
    let atk = g.new_fx(Effect::Attack { p: p as u8, opp: o as u8, attack, damage: 0, ignore_weakness: false, ignore_resistance: false, ignore_defender_effects: false, source, barrage_used: false });
    let b = AtkBase { attack_effect: atk, player: p as u8, opponent: o as u8, attack, source, target, cause };
    let r = g.run_fx(Effect::DiscardCards { b, cards: SVec::new() });
    g.release_fx(atk);
    Ok(r?.1)
}
