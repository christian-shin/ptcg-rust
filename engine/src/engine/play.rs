//! `play-trainer-effect.ts` (playing a Pokémon card: `engine::enter`; attaching an Energy or a Tool: `engine::attach`).

use crate::effects::*;
use crate::game::{Cont, Game, R};
use crate::list::*;
use crate::spec::passive::{lasting_locked, LockedAction};
use crate::state::*;
use crate::types::*;

fn cleanup_target(g: &Game, card: CardId) -> fn(u8) -> ListRef {
    if g.st.cdef(card).has_tag(tag::PRISM_STAR) {
        ListRef::LostZone
    } else {
        ListRef::Discard
    }
}

pub fn trainer_cleanup(g: &mut Game, p: usize, card: CardId) {
    if !g.st.players[p].supporter.contains(card) {
        return;
    }
    let t = cleanup_target(g, card)(p as u8);
    g.move_card_to(ListRef::Supporter(p as u8), card, t);
}

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

/// An Item being played leaves the hand for the play area before its effect runs (legality evaluates the
/// card's declared checks in the same state).
pub fn enter_item_play(g: &mut Game, p: usize, card: CardId) {
    g.move_card_to(ListRef::Hand(p as u8), card, ListRef::Supporter(p as u8));
}

/// Which `playTrainerReducer` branch a Seismitoad-style coin flip resumes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrainerPlayKind {
    Supporter,
    Stadium,
    Tool,
    Item,
}

/// `withOptionalCoinFlipCancelTrainer`: with the flag set, flip a coin
/// (CoinFlipEffect) and resume in [`cancel_trainer_coin`]; else (or in a
/// legality trial) continue now.
fn with_optional_coin_flip_cancel_trainer(g: &mut Game, kind: TrainerPlayKind, p: u8, card: CardId, target: Option<SlotRef>) -> R {
    if g.st.players[p as usize].coin_flip_cancel_trainer_play_turns_remaining <= 0 {
        return continue_trainer_play(g, kind, p, card, target);
    }
    // Legality trial (`Chance.inTrial`): a fixed tails would discard the card
    // without running its checks, so check the heads path instead.
    if g.rng.is_fixed() {
        return continue_trainer_play(g, kind, p, card, target);
    }
    // The flip is an effect of the opponent's attack (Seismitoad's Quaking Fist), whose card isn't kept.
    let cause = crate::cause::Cause::new(crate::cause::CauseKind::Attack, None, 1 - p);
    g.coin_flip(p as usize, crate::game::CoinCb::CancelTrainer { kind, p, card, target }, cause)?;
    Ok(())
}

/// The CoinFlipEffect callback of `withOptionalCoinFlipCancelTrainer`.
pub fn cancel_trainer_coin(g: &mut Game, kind: TrainerPlayKind, p: u8, card: CardId, target: Option<SlotRef>, heads: bool) -> R {
    if heads {
        return continue_trainer_play(g, kind, p, card, target);
    }
    let pu = p as usize;
    let t = cleanup_target(g, card)(p);
    if g.st.players[pu].hand.contains(card) {
        g.move_card_to(ListRef::Hand(p), card, t);
    } else if g.st.players[pu].supporter.contains(card) {
        g.move_card_to(ListRef::Supporter(p), card, t);
    }
    Ok(())
}

fn continue_trainer_play(g: &mut Game, kind: TrainerPlayKind, p: u8, card: CardId, target: Option<SlotRef>) -> R {
    let pu = p as usize;
    match kind {
        TrainerPlayKind::Supporter => {
            g.run_fx_unit(Effect::Trainer { p, card, target, via_attack: false })?;
            // `rocketSupporter` (read by Team Rocket's Factory and Kangaskhan ex): a Team Rocket's
            // Supporter played from the hand (K1; a Supporter's effect used by an attack never gets here).
            if g.st.cdef(card).has_tag(tag::TEAM_ROCKET) {
                g.st.players[pu].rocket_supporter = true;
            }
            restore_played_trainer(g, pu, card);
            let keep = g.st.rules.supporter_cleanup_at_end_turn;
            finalize_trainer_cleanup(g, pu, card, keep);
            g.st.players[pu].supporter_turn += 1;
            Ok(())
        }
        TrainerPlayKind::Stadium => {
            let stadium = g.st.stadium_card();
            let prism = stadium.map(|s| g.st.cdef(s).has_tag(tag::PRISM_STAR)).unwrap_or(false);
            for q in [pu, 1 - pu] {
                if !g.st.players[q].stadium.is_empty() {
                    let dst = if prism { ListRef::LostZone(q as u8) } else { ListRef::Discard(q as u8) };
                    g.move_to(ListRef::Stadium(q as u8), dst, None);
                }
            }
            g.st.players[pu].stadium_used_turn = 0;
            g.move_card_to(ListRef::Hand(p), card, ListRef::Stadium(p));
            Ok(())
        }
        TrainerPlayKind::Tool => {
            let target = match target {
                Some(t) => t,
                None => return Ok(()),
            };
            // The Tool is attached from the hand (the Attach event, checked by `can_attach_tool`), then its Trainer
            // effect.
            crate::engine::attach::run_attach(g, card, target, false, crate::cause::Cause::rule(crate::cause::RuleWhich::Action, p))?;
            g.run_fx_unit(Effect::Trainer { p, card, target: Some(target), via_attack: false })?;
            Ok(())
        }
        TrainerPlayKind::Item => {
            enter_item_play(g, pu, card);
            g.run_fx_unit(Effect::Trainer { p, card, target, via_attack: false })?;
            restore_played_trainer(g, pu, card);
            finalize_trainer_cleanup(g, pu, card, false);
            Ok(())
        }
    }
}

/// Check only: the Trainer-play rules the `play_trainer_reducer` applies.
pub fn can_play_supporter(g: &Game, p: usize) -> R {
    can_play_supporter_with(g, p, None)
}

/// [`can_play_supporter`] for this card (`None`: any Supporter): the locks an attack left judge the card.
pub fn can_play_supporter_with(g: &Game, p: usize, card: Option<CardId>) -> R {
    if let Some(code) = lasting_locked(g, p, card, &[LockedAction::PlaySupporter]) {
        crate::bail!(code);
    }
    // One Supporter card per turn (basic rule), for every Supporter.
    if g.st.players[p].supporter_turn > 0 {
        crate::bail!("SUPPORTER_ALREADY_PLAYED");
    }
    Ok(())
}

pub fn can_play_stadium(g: &Game, p: usize) -> R {
    can_play_stadium_with(g, p, None)
}

/// [`can_play_stadium`] for this card (`None`: any Stadium).
pub fn can_play_stadium_with(g: &Game, p: usize, card: Option<CardId>) -> R {
    if let Some(code) = lasting_locked(g, p, card, &[LockedAction::PlayStadium]) {
        crate::bail!(code);
    }
    Ok(())
}

pub fn can_play_item(g: &Game, p: usize) -> R {
    can_play_item_with(g, p, None)
}

/// [`can_play_item`] for this card (`None`: any Item).
pub fn can_play_item_with(g: &Game, p: usize, card: Option<CardId>) -> R {
    if let Some(code) = lasting_locked(g, p, card, &[LockedAction::PlayItem]) {
        crate::bail!(code);
    }
    Ok(())
}

/// The checks of playing the Tool `card` from the hand onto `target`: the Attach it produces
/// (`engine::attach::check_tool_play`: the spot, one Tool per Pokémon, the locks).
pub fn can_attach_tool(g: &mut Game, p: usize, card: CardId, target: SlotRef) -> R {
    crate::engine::attach::check_tool_play(g, p, card, target)
}

pub fn play_trainer_reducer(g: &mut Game, id: EffId) -> R {
    match *g.e(id) {
        Effect::PlaySupporter { p, card, target } => {
            let pu = p as usize;
            can_play_supporter_with(g, pu, Some(card))?;
            with_optional_coin_flip_cancel_trainer(g, TrainerPlayKind::Supporter, p, card, target)
        }
        Effect::PlayStadium { p, card } => {
            let pu = p as usize;
            can_play_stadium_with(g, pu, Some(card))?;
            with_optional_coin_flip_cancel_trainer(g, TrainerPlayKind::Stadium, p, card, None)
        }
        Effect::AttachPokemonTool { p, card, target } => {
            let pu = p as usize;
            can_attach_tool(g, pu, card, target)?;
            with_optional_coin_flip_cancel_trainer(g, TrainerPlayKind::Tool, p, card, Some(target))
        }
        Effect::PlayItem { p, card, target } => {
            let pu = p as usize;
            can_play_item_with(g, pu, Some(card))?;
            with_optional_coin_flip_cancel_trainer(g, TrainerPlayKind::Item, p, card, target)
        }
        Effect::Trainer { p, card, .. } => {
            if g.st.players[p as usize].hand.contains(card) {
                let dst = if g.st.cdef(card).trainer_type == TrainerType::Supporter as u8 {
                    ListRef::Supporter(p)
                } else {
                    ListRef::Discard(p)
                };
                g.move_card_to(ListRef::Hand(p), card, dst);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
