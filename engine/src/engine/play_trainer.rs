//! The PlayTrainer event of events batch 7 (docs/design/events-design.md, section 4; APR B-01..B-04, C-19, E-26, E-29):
//! a Trainer card played from the hand by the game rule (`TrainerUse::Played`), or its effect used by another card
//! without playing it (`TrainerUse::Used`: Mr. Mime's Look-Alike Show; id2225, id2226, id2376).
//!
//! In the `engine::enter` pattern: one check function ([`check_with`], which execution and legality, `legal.rs
//! fast_trainer`, both call): the turn's rules for a played card (one Supporter per turn, not on the first player's first
//! turn unless the card says so; one Stadium per turn, not the name of the one in play; a Tool: the Attach it produces),
//! then the locks over PlayTrainer (`derived::event_locked`: "your opponent can't play Item cards from their hand" is
//! `Kind(PlayTrainer) & Use(Played) & Source(Hand) & Card(Item)`; a lasting one on the actor). A used card keeps only its
//! printed conditions (checked by its program, `spec::run::check_play`), none of the play rules.
//!
//! [`play_from_hand`] is the turn action: the checks, Seismitoad's coin gate (a lasting lock with a coin, `CoinGate`:
//! "whenever they try to play a Trainer card from their hand, they flip a coin; if tails, they discard that card instead";
//! not a legality check: heads may succeed), then the play: the card goes to the play area (an Item, a Supporter) or
//! into play (a Stadium replacing the old one, whose LeavePlay is the rule's; a Tool's Attach), the event is produced (the
//! Trainer's own program runs as its handler: `spec::run::reduce`), and the card is discarded after use (a consequence of
//! the event, not a Discard event: user decision D13; a Supporter at the end of the turn). The played-this-turn record
//! (`Player::played_this_turn`) is the one source of "if you played a Supporter card from your hand this turn" (E-26).

use crate::cause::{Cause, RuleWhich};
use crate::effects::{Effect, SlotRef};
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::spec::event::{EventKind, EventView, RulesZone, TrainerUse};
use crate::state::ListRef;
use crate::types::*;

/// The PlayTrainer event of `card` (`use_`, from `source`), as predicates read it. The event's owner is the card's owner.
pub fn play_view(g: &Game, card: CardId, use_: TrainerUse, source: RulesZone, cause: Cause) -> EventView {
    EventView { card: Some(card), source: Some(source), trainer_use: Some(use_), ..EventView::new(EventKind::PlayTrainer, cause, g.st.owner(card) as u8, crate::spec::event::whose_turn(g)) }
}

/// Has player `p` played a card matching `pred` from their hand this turn (the played-this-turn record; APR E-26)?
pub fn played_this_turn(g: &Game, p: usize, f: impl Fn(&'static crate::carddb::CardDef) -> bool) -> bool {
    g.st.players[p].played_this_turn.iter().any(|c| f(g.st.cdef(*c)))
}

/// A card played from the hand this turn joins the record (cleared at the end of its player's turn).
fn record(g: &mut Game, p: usize, card: CardId) {
    let r = &mut g.st.players[p].played_this_turn;
    if r.len() < r.capacity() {
        r.push(card);
    }
}

// ---------------------------------------------------------------------------
// The checks (execution and legality call the same function)

/// Where the checks of a PlayTrainer read the game: execution on the game, legality on its scratch game behind its
/// plain-read gate. Both answer through [`check_with`].
pub trait PlayChecks {
    fn game(&self) -> &Game;
    /// The one lock query (`derived::event_locked`): the code of the lock that forbids the event.
    fn event_locked(&mut self, v: &EventView) -> R<Option<&'static str>>;
    /// The checks of the Attach a Tool play produces (`engine::attach::check_attach_with`).
    fn tool_attach(&mut self, card: CardId, target: SlotRef) -> R<Option<&'static str>>;
}

impl PlayChecks for Game {
    fn game(&self) -> &Game {
        self
    }
    fn event_locked(&mut self, v: &EventView) -> R<Option<&'static str>> {
        crate::derived::event_locked(self, v)
    }
    fn tool_attach(&mut self, card: CardId, target: SlotRef) -> R<Option<&'static str>> {
        let v = crate::engine::attach::attach_view(self, card, target, RulesZone::Hand, false, Cause::rule(RuleWhich::Action, self.st.owner(card) as u8));
        crate::engine::attach::check_attach_with(self, &v)
    }
}

/// The turn's rules for playing `card` from the hand (a plain read): a Supporter not on the first player's first turn
/// (unless the card says so, `CardDef::first_turn`) and one per turn; a Stadium one per turn and not with the name of the
/// Stadium in play (APR B-03, B-04).
pub fn turn_rules(g: &Game, p: usize, card: CardId) -> Option<&'static str> {
    let d = g.st.cdef(card);
    match d.trainer_type() {
        TrainerType::Supporter => {
            if g.st.turn == 1 && !d.first_turn {
                return Some("CANNOT_PLAY_THIS_CARD");
            }
            if played_this_turn(g, p, |c| c.trainer_type == TrainerType::Supporter as u8) {
                return Some("SUPPORTER_ALREADY_PLAYED");
            }
        }
        TrainerType::Stadium => {
            if played_this_turn(g, p, |c| c.trainer_type == TrainerType::Stadium as u8) {
                return Some("STADIUM_ALREADY_PLAYED");
            }
            if let Some(s) = g.st.stadium_card() {
                if g.st.cdef(s).name == d.name {
                    return Some("SAME_STADIUM_ALREADY_IN_PLAY");
                }
            }
        }
        _ => {}
    }
    None
}

/// Every check of playing `card` from the hand (`v`, a `Played` view), in order: the turn's rules, a Tool's Attach (the
/// spot, one Tool per Pokémon, the Attach's locks), the locks over PlayTrainer (a coin-gated lock isn't one: it may let
/// the play through). `Ok(Some(code))` when the play is refused (the action is illegal).
pub fn check_with<C: PlayChecks + ?Sized>(c: &mut C, v: &EventView, target: Option<SlotRef>) -> R<Option<&'static str>> {
    let (Some(card), Some(TrainerUse::Played)) = (v.card, v.trainer_use) else { return Ok(None) };
    let p = v.actor() as usize;
    if let Some(code) = turn_rules(c.game(), p, card) {
        return Ok(Some(code));
    }
    if c.game().st.cdef(card).trainer_type() == TrainerType::Tool {
        let Some(t) = target else { return Ok(Some("INVALID_TARGET")) };
        if let Some(code) = c.tool_attach(card, t)? {
            return Ok(Some(code));
        }
    }
    c.event_locked(v)
}

// ---------------------------------------------------------------------------
// The routine

/// The player's turn action: play the Trainer `card` from the hand (`target`: a Tool's Pokémon). Refused plays are errors
/// (the action is illegal).
pub fn play_from_hand(g: &mut Game, p: usize, card: CardId, target: Option<SlotRef>) -> R {
    let cause = Cause::rule(RuleWhich::Action, p as u8);
    let v = play_view(g, card, TrainerUse::Played, RulesZone::Hand, cause);
    if let Some(code) = check_with(g, &v, target)? {
        crate::bail!(code);
    }
    // A coin-gated lock (Seismitoad's Quaking Fist): the coin is flipped first, before the card's own costs and before
    // the old Stadium is discarded (JP FAQ ガマゲロゲ). A legality trial checks the heads path.
    if let Some(gate_cause) = crate::spec::passive::coin_gate(g, &v)? {
        if !g.rng.is_fixed() {
            g.coin_flip(p, crate::game::CoinCb::PlayGate { p: p as u8, card, target }, gate_cause)?;
            return Ok(());
        }
    }
    resolve(g, p, card, target)
}

/// The coin of a coin-gated lock (`CoinCb::PlayGate`): heads, the card is played; tails, the play doesn't happen (it
/// isn't recorded: the same Stadium may be played again, another Supporter may be played; JP FAQ ガマゲロゲ + ミアレシティ)
/// and the card is discarded instead (a Discard by the lock's cause).
pub fn gate_coin(g: &mut Game, p: u8, card: CardId, target: Option<SlotRef>, heads: bool) -> R {
    if heads {
        return resolve(g, p as usize, card, target);
    }
    let cause = crate::spec::passive::coin_gate_cause(g, p as usize).unwrap_or(Cause::rule(RuleWhich::Action, p));
    for from in [ListRef::Hand(p), ListRef::Supporter(p)] {
        if g.lst(from).contains(&card) {
            crate::engine::cards_zone::discard(g, from, &[card], cause)?;
        }
    }
    Ok(())
}

/// The play itself, checks done: the card's way into the play area or into play, the event (its program), the cleanup.
fn resolve(g: &mut Game, p: usize, card: CardId, target: Option<SlotRef>) -> R {
    let pu = p as u8;
    let cause = Cause::rule(RuleWhich::Action, pu);
    match g.st.cdef(card).trainer_type() {
        TrainerType::Supporter => {
            g.run_fx_unit(Effect::PlayTrainer { p: pu, card, target, use_: TrainerUse::Played, cause })?;
            restore_played_trainer(g, p, card);
            let keep = g.st.rules.supporter_cleanup_at_end_turn;
            finalize_trainer_cleanup(g, p, card, keep);
            record(g, p, card);
            Ok(())
        }
        TrainerType::Stadium => {
            record(g, p, card);
            // The old Stadium leaves play by the rule (APR B-04; user decision D1: a LeavePlay, never refused).
            if let Some(old) = g.st.stadium_card() {
                crate::engine::knockout::leave_play_stadium_by_rule(g, old, cause)?;
            }
            g.st.players[p].stadium_used_turn = 0;
            g.move_card_to(ListRef::Hand(pu), card, ListRef::Stadium(pu));
            g.run_fx_unit(Effect::PlayTrainer { p: pu, card, target: None, use_: TrainerUse::Played, cause })?;
            // A Stadium coming into play can hold or lift an Ability lock.
            crate::engine::cards_zone::settle(g);
            Ok(())
        }
        TrainerType::Tool => {
            let Some(t) = target else { return Ok(()) };
            // The Tool is attached from the hand (the Attach event, checked with the play), then the event.
            crate::engine::attach::run_attach(g, card, t, false, cause)?;
            g.run_fx_unit(Effect::PlayTrainer { p: pu, card, target: Some(t), use_: TrainerUse::Played, cause })?;
            crate::engine::cards_zone::settle(g);
            Ok(())
        }
        TrainerType::Item => {
            enter_item_play(g, p, card);
            g.run_fx_unit(Effect::PlayTrainer { p: pu, card, target, use_: TrainerUse::Played, cause })?;
            restore_played_trainer(g, p, card);
            finalize_trainer_cleanup(g, p, card, false);
            record(g, p, card);
            Ok(())
        }
    }
}

/// Use the effect of the Trainer `card` (wherever it is: the opponent's hand for Look-Alike Show) as the effect of `cause`'s
/// attack (`TrainerUse::Used`): no play rule applies, the card doesn't move, its program keeps its printed conditions.
pub fn use_effect(g: &mut Game, p: usize, card: CardId, cause: Cause) -> R {
    g.run_fx_unit(Effect::PlayTrainer { p: p as u8, card, target: None, use_: TrainerUse::Used, cause })
}

/// An Item being played leaves the hand for the play area before its effect runs (legality evaluates the card's declared
/// checks in the same state).
pub fn enter_item_play(g: &mut Game, p: usize, card: CardId) {
    g.move_card_to(ListRef::Hand(p as u8), card, ListRef::Supporter(p as u8));
}

fn cleanup_target(g: &Game, card: CardId) -> fn(u8) -> ListRef {
    if g.st.cdef(card).has_tag(tag::PRISM_STAR) {
        ListRef::LostZone
    } else {
        ListRef::Discard
    }
}

/// After its use the played card goes from the play area to its owner's discard pile (a Prism Star card to the Lost
/// Zone): a consequence of the play (user decision D13).
pub fn trainer_cleanup(g: &mut Game, p: usize, card: CardId) {
    if !g.st.players[p].supporter.contains(card) {
        return;
    }
    let t = cleanup_target(g, card)(p as u8);
    g.move_card_to(ListRef::Supporter(p as u8), card, t);
}

/// A played card its program discarded while prompts are still open goes back to the play area until the effect ends.
fn restore_played_trainer(g: &mut Game, p: usize, card: CardId) {
    if !g.has_prompts() {
        return;
    }
    if g.st.players[p].discard.contains(card) {
        g.move_card_to(ListRef::Discard(p as u8), card, ListRef::Supporter(p as u8));
    }
}

fn finalize_trainer_cleanup(g: &mut Game, p: usize, card: CardId, keep: bool) {
    if keep {
        return;
    }
    if g.has_prompts() {
        g.wait_prompt(Cont::TrainerCleanup { p: p as u8, card });
        return;
    }
    trainer_cleanup(g, p, card);
}

/// The PlayTrainer's consequence the program sees: a played Supporter goes from the hand to the play area (its program
/// runs first, as its handler; a used card stays where it is).
pub fn reducer(g: &mut Game, id: crate::effects::EffId) -> R {
    if let Effect::PlayTrainer { p, card, use_: TrainerUse::Played, .. } = *g.e(id) {
        if g.st.players[p as usize].hand.contains(card) {
            let dst = if g.st.cdef(card).trainer_type == TrainerType::Supporter as u8 { ListRef::Supporter(p) } else { ListRef::Discard(p) };
            g.move_card_to(ListRef::Hand(p), card, dst);
        }
    }
    Ok(())
}
