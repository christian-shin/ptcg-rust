//! Helpers shared by card programs and the engine. Helpers that open prompts continue through [`PrefabCont`], so
//! callers can reuse them without their own frames.

use crate::effects::*;
use crate::game::{CoinCb, Cont, Game, R};
use crate::list::*;
use crate::prompts::*;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, Debug)]
pub enum PrefabCont {
    /// "Shuffle your hand into your deck, then draw": the hand is in the deck and its wait is done; shuffle, then draw.
    /// `after`: a card continuation resumed with no results after the draw; `cause` packed (`Cause::pack`).
    ShuffleThenDraw { p: u8, draw: u8, after: Option<(CardId, crate::cards::CardFrame)>, cause: [u8; 4] },
    /// Shuffle order chosen: apply, then wait, then draw.
    ShuffleOrderThenDraw { p: u8, draw: u8, after: Option<(CardId, crate::cards::CardFrame)>, cause: [u8; 4] },
    DrawAfterWait { p: u8, draw: u8, after: Option<(CardId, crate::cards::CardFrame)>, cause: [u8; 4] },
    /// THIS_ATTACK_DOES_X_DAMAGE_TO_1_OF_YOUR_OPPONENTS_[BENCHED_]POKEMON.
    DamageChosen { atk: EffId, damage: i32 },
}

pub fn resume(g: &mut Game, c: PrefabCont, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    match c {
        PrefabCont::ShuffleThenDraw { p, draw, after, cause } => {
            shuffle_then_draw(g, p as usize, draw, after, cause);
            Ok(())
        }
        PrefabCont::ShuffleOrderThenDraw { p, draw, after, cause } => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.zone_edit(ListRef::Deck(p)), o.as_slice());
                g.st.bump_layout();
            }
            g.wait(p as usize, Cont::Prefab(PrefabCont::DrawAfterWait { p, draw, after, cause }));
            Ok(())
        }
        PrefabCont::DrawAfterWait { p, draw, after, cause } => {
            crate::engine::cards_zone::draw(g, p as usize, draw as usize, crate::cause::Cause::unpack(cause))?;
            if let Some((card, frame)) = after {
                crate::cards::resume(g, card, frame, &[])?;
            }
            Ok(())
        }
        PrefabCont::DamageChosen { atk, damage } => {
            let sel = match first {
                Res::Slots(s) => s,
                // Not cancellable, so a result is always given.
                _ => return Ok(()),
            };
            let t = match sel.get(0) {
                Some(t) => *t,
                // A required choice answered with nothing: the answer is refused (the legality layer never offers it).
                None => crate::bail!("INVALID_PROMPT_RESULT"),
            };
            let r = deal_or_put_damage(g, atk, damage, t);
            g.release_fx(atk);
            r
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

/// Whether the effect is the use of this card's attack number `index`.
///
/// While a copy-attack session runs a source handler for the copycat, the
/// copycat's attacks are the session's clones.
pub fn was_attack_used(g: &Game, e: EffId, index: u8, me: CardId) -> bool {
    let mine = my_attack(g, me, index);
    matches!(*g.e(e), Effect::Attack { attack, .. } if attack == mine)
}

/// "Remaining HP becomes 10" on a coin flip: when the damage would Knock Out
/// its target (existing damage plus the damage >= its HP) during an attack, the Pokémon is recorded:
/// the full damage is done, then the coin is flipped and 10 HP restored AFTER all the damage (ruling 1770,
/// `resolve_survive_coin_flips`). Outside an attack the coin is flipped right away (no
/// callback: a callback would run after the flip's wait prompt, i.e. after the damage was applied); heads records the
/// survival.
pub fn survive_on_ten_on_coin_flip(g: &mut Game, acc: &mut crate::spec::passive::DamageAcc, player: usize, cause: crate::cause::Cause) -> R {
    let (t, damage) = (acc.b.target, acc.damage);
    let hp = crate::derived::hp(g, player, t.s)?;
    if g.st.slot(t.p as usize, t.s).damage + damage >= hp {
        if g.st.phase == GamePhase::Attack {
            if !g.ten_hp_coin.iter().any(|(s, _)| *s == t) {
                g.ten_hp_coin.push((t, player as u8));
            }
            return Ok(());
        }
        if crate::engine::condition::flip_coin(g, player, CoinCb::None, cause)? {
            acc.survive = true;
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
        let hp = crate::derived::hp(g, owner as usize, ts)?;
        if g.st.slot(tp, ts).damage < hp {
            continue;
        }
        // The Pokémon's own Ability ("flip a coin; if heads, it is not Knocked Out").
        let cause = crate::cause::Cause::new(crate::cause::CauseKind::Ability, g.st.slot_pokemon(tp, ts), owner);
        if crate::engine::condition::flip_coin(g, owner as usize, CoinCb::None, cause)? {
            crate::engine::damage::survive_on_10(g, t, hp);
        }
    }
    Ok(())
}

/// `AFTER_ATTACK(effect, index, this)`.
pub fn after_attack_used(g: &Game, e: EffId, index: u8, me: CardId) -> bool {
    let mine = my_attack(g, me, index);
    matches!(*g.e(e), Effect::AfterAttack { attack, .. } if attack == mine)
}

/// The attack's own AttackEffect behind an AfterAttack effect; any other effect is returned
/// as is. A block moved to AfterAttack (effect text asked after the damage) calls this first and then reads the
/// attack's state as before.
pub fn real_attack(g: &Game, e: EffId) -> EffId {
    match *g.e(e) {
        Effect::AfterAttack { atk, .. } => atk,
        _ => e,
    }
}

/// (player, opponent, attack, source) of an attack effect, or of
/// the attack a card builds in an AfterAttack handler (its source is the player's Active).
pub fn attack_data(g: &Game, e: EffId) -> Option<(u8, u8, AttackRef, SlotRef)> {
    if e as usize >= g.fx.len() {
        return None;
    }
    match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => Some((p, opp, attack, source)),
        Effect::AfterAttack { p, opp, attack, .. } => Some((p, opp, attack, SlotRef::new(p as usize, g.st.players[p as usize].active))),
        _ => None,
    }
}

/// `WAS_POWER_USED(effect, index, this)`.
pub fn was_power_used(g: &Game, e: EffId, index: u8, me: CardId) -> bool {
    matches!(*g.e(e), Effect::Power { power, .. } if power == PowerRef { card: me, index })
}

/// The state a Pokémon keeps when a card replaces another: a Pokémon switched with another card
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

/// The PlayTrainer event uses the Supporter's effect as the effect of an attack (`TrainerUse::Used`: Mr. Mime's Look-Alike
/// Show), so "up to" prompts may choose zero (APR D-05).
pub fn trainer_via_attack(g: &Game, e: EffId) -> bool {
    matches!(*g.e(e), Effect::PlayTrainer { use_: crate::spec::event::TrainerUse::Used, .. })
}

/// The PlayTrainer event of `me` (played or used): the player whose program it is.
pub fn trainer_played(g: &Game, e: EffId, me: CardId) -> Option<usize> {
    match *g.e(e) {
        Effect::PlayTrainer { p, card, .. } if card == me => Some(p as usize),
        _ => None,
    }
}

/// The slots with a Pokémon card, Active first, with their targets.
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

/// The Energy
/// cards attached to the slot that provide `ty` as the Pokémon's Energy provides it right now
/// (the Energy the Pokémon provides); an Energy that provides every type (Legacy Energy, Prism Energy on a Basic Pokémon)
/// is a [R] Energy, a [W] Energy, ... for every "[X] Energy" in card text (Advanced Rulebook D-08).
pub fn energy_cards_that_provide_type(g: &mut Game, p: usize, s: SlotId, ty: CardType) -> R<SVec<CardId, 64>> {
    let energy_map = crate::derived::provided_energy(g, p, SlotRef::new(p, s))?;
    let mut cards: SVec<CardId, 64> = SVec::new();
    for m in energy_map.iter() {
        if (m.provides.contains(&ty) || m.provides.contains(&ct::ANY)) && !cards.contains(&m.card) {
            cards.push(m.card);
        }
    }
    Ok(cards)
}

/// The prompt's blocked indices of the slot's Energy cards that do not provide
/// `ty`; `None` when there is none.
pub fn blocked_non_type_energy(g: &mut Game, p: usize, s: SlotId, ty: CardType) -> R<Option<Blocked>> {
    let providing = energy_cards_that_provide_type(g, p, s, ty)?;
    let mut b = Blocked::default();
    let mut any = false;
    for (i, c) in g.st.slot(p, s).cards().iter().enumerate() {
        if g.st.cdef(c).is_energy() && !providing.contains(&c) {
            b.push(i as u8);
            any = true;
        }
    }
    Ok(if any { Some(b) } else { None })
}

// ---------------------------------------------------------------------------
// Card movement (the events are `engine::cards_zone`'s and `engine::knockout`'s)

/// A shuffle of the deck that waits for the attack's damage.
pub fn shuffle_deck_after_damage(g: &mut Game, atk: EffId, p: usize) {
    if g.after_damage_open(atk) {
        g.push_after_damage(atk, crate::game::AfterDmgStep::Shuffle(p as u8));
    } else {
        shuffle_deck(g, p);
    }
}

/// The Shuffle routine (`engine::cards_zone::shuffle_deck`): the ShuffleDeck prompt, then a silent wait.
pub fn shuffle_deck(g: &mut Game, p: usize) {
    crate::engine::cards_zone::shuffle_deck(g, p);
}

fn shuffle_then_draw(g: &mut Game, p: usize, draw: u8, after: Option<(CardId, crate::cards::CardFrame)>, cause: [u8; 4]) {
    g.prompt(p, PromptPurpose::Shuffle, PromptKind::ShuffleDeck, Provenance::NONE, Cont::Prefab(PrefabCont::ShuffleOrderThenDraw { p: p as u8, draw, after, cause }));
}

/// "Shuffle your hand into your deck. Then, draw N cards" (Judge, Lacey, Lillie's Determination, ...): the hand (without
/// `exclude`, the resolving card) is put into the deck (a PutIntoDeck, `ShuffledIn`), then the deck is shuffled (the
/// ShuffleDeck prompt after the move's wait), then the Draw; `after` is resumed after the draw. With an empty hand the
/// shuffle and the draw still happen.
pub fn shuffle_hand_into_deck_then_draw(g: &mut Game, p: usize, exclude: CardId, draw: u8, after: Option<(CardId, crate::cards::CardFrame)>, cause: crate::cause::Cause) -> R {
    let cards: SVec<CardId, 64> = {
        let mut v = SVec::new();
        for c in g.st.players[p].hand().iter() {
            if c != exclude {
                v.push(c);
            }
        }
        v
    };
    if !cards.is_empty() {
        crate::engine::cards_zone::put_into_deck(g, ListRef::Hand(p as u8), cards.as_slice(), crate::spec::event::DeckPosition::ShuffledIn, cause)?;
        g.wait(p, Cont::Prefab(PrefabCont::ShuffleThenDraw { p: p as u8, draw, after, cause: cause.pack() }));
        return Ok(());
    }
    shuffle_then_draw(g, p, draw, after, cause.pack());
    Ok(())
}

/// `CONFIRMATION_PROMPT(store, state, player, callback, message)`.
pub fn confirmation_prompt(g: &mut Game, p: usize, purpose: PromptPurpose, provenance: Provenance, cont: Cont) {
    g.prompt(p, purpose, PromptKind::Confirm, provenance, cont);
}

/// Asks the player to choose cards from `list`; the prompt lists the zone in its own order (the deck in deck order, the
/// discard pile oldest first).
pub fn choose_cards(g: &mut Game, p: usize, purpose: PromptPurpose, list: ListRef, filter: Filter, opts: ChooseCardsOpts, provenance: Provenance, cont: Cont) {
    g.prompt(p, purpose, PromptKind::ChooseCards { cards: list, filter, opts }, provenance, cont);
}

// ---------------------------------------------------------------------------
// Probes

/// Power flags carried by a lock probe (`IS_ABILITY_BLOCKED`'s stand-in power).
/// `index == PROBE_GENERIC` means a plain Ability with no flags.
pub const PROBE_GENERIC: u8 = 255;

// ---------------------------------------------------------------------------
// Attack helpers

fn atk_base_for(g: &Game, atk: EffId, target: SlotRef) -> AtkBase {
    match *g.e(atk) {
        Effect::Attack { p, opp, attack, source, .. } => AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target, cause: crate::cause::Cause::of_attack_at(g, p, attack, source) },
        _ => panic!("not an attack effect"),
    }
}

/// The attack's damage to `target` (the Damage event, `engine::damage::deal`): dealt to the opponent's Active Pokémon,
/// put on the others.
pub fn deal_or_put_damage(g: &mut Game, atk: EffId, damage: i32, target: SlotRef) -> R {
    let b = atk_base_for(g, atk, target);
    let o = b.opponent as usize;
    let deal = target.p as usize == o && target.s == g.st.players[o].active;
    crate::engine::damage::deal(g, b, damage, deal)
}

/// The attack's damage put on `target` (the Damage event without the attacker-side passes).
pub fn put_damage(g: &mut Game, atk: EffId, damage: i32, target: SlotRef) -> R {
    let b = atk_base_for(g, atk, target);
    crate::engine::damage::deal(g, b, damage, false)
}

// ---------------------------------------------------------------------------
// Deck search

/// `GET_PLAYER_BENCH_SLOTS`: empty bench slots in order.
pub fn empty_bench_slots(g: &Game, p: usize) -> SVec<SlotId, 8> {
    let pl = &g.st.players[p];
    let mut v = SVec::new();
    for &b in pl.bench.iter() {
        if pl.slots[b as usize].cards().is_empty() {
            v.push(b);
        }
    }
    v
}

/// `MULTIPLE_COIN_FLIPS_PROMPT` / `FLIP_UNTIL_TAILS` (`mode` 0 = until tails):
/// the callback receives the results as a bitmask (bit i = flip i heads) and count.
pub fn coin_flip_sequence(g: &mut Game, p: usize, mode: u8, cb: CoinCb, cause: crate::cause::Cause) -> R {
    crate::engine::condition::flip_sequence(g, p, mode, cb, cause)
}

/// Whether the damage ignores the effects on the Defending Pokémon: Shred ("isn't affected by any effects on your opponent's Active
/// Pokémon"). True for damage by an attack that ignores defender effects, done to one
/// of the opponent's Pokémon. Effects on the damaged Pokémon (prevention, reduction, extra damage taken,
/// Tera/Bench protection) must not change it (rulings 1439, 1345, 1490, 1629, 1875); effects on the
/// attacker, Weakness/Resistance and survive-on-10-HP effects (rulings 936, 1770) still apply.
pub fn ignores_defender_effects(g: &Game, b: &AtkBase) -> bool {
    matches!(*g.e(b.attack_effect), Effect::Attack { ignore_defender_effects: true, .. }) && b.target.p == b.opponent
}
