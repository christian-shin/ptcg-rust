//! Mr. Mime (TEF): Look-Alike Show — your opponent reveals their hand; you may
//! use the effect of a Supporter card you find there as the effect of this
//! attack. Eerie Wave — 20; your opponent's Active Pokémon is now Confused.
//!
//! Twinleaf: Look-Alike Show throws CANNOT_USE_POWER when this Pokémon is not
//! the Active, then opens a ChooseCardsPrompt on the opponent's hand
//! (Supporters, 0..1, no cancel); the callback reduces a TrainerEffect for the
//! chosen card with the attacker as player.
//! Fixed (phase 4b): a Special Condition no longer stops the attack (it threw
//! CANNOT_USE_POWER, ending the game after a heads Confusion flip); using the
//! effect of a Supporter is not playing it, so `supporterTurn` is bypassed
//! (SUPPORTER_ALREADY_PLAYED); a Supporter whose effect throws is blocked in
//! the prompt, which is re-issued (CANNOT_PLAY_THIS_CARD).
//!
//! R7C: the TrainerEffect carries `via_attack` (Twinleaf `usedAsAttackEffect`): the
//! Supporter's "up to" prompts may choose zero and the played-from-hand trackers
//! (`rocketSupporter`, `ancientSupporter`) are not set (rulings 1727, 1844, 1853).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MrMime", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, opp) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        match g.st.locate(me) {
            Some(ListRef::Slot(q, s)) => {
                if !(q as usize == p && s == g.st.players[p].active) {
                    bail!("CANNOT_USE_POWER");
                }
            }
            _ => bail!("CANNOT_USE_POWER"),
        }
        return choose_supporter(g, me, p, opp, Blocked::default());
    }

    if was_attack_used(g, e, 1, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Confused])?;
    }
    Ok(())
}

/// `chooseSupporter(blocked)`: the ChooseCardsPrompt on the opponent's hand.
fn choose_supporter(g: &mut Game, me: CardId, p: usize, opp: usize, blocked: Blocked) -> R {
    let mut filter = Filter::super_type(SuperType::Trainer);
    filter.trainer_type = Some(TrainerType::Supporter as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = (blocked.0 & 0xFFFF_FFFF) as u32 as i32;
    f.a[2] = (blocked.0 >> 32) as u32 as i32;
    let mut opts = ChooseCardsOpts::new(0, 1, false);
    opts.blocked = blocked;
    choose_cards(g, p, "CHOOSE_CARD_TO_COPY_EFFECT", ListRef::Hand(opp as u8), filter, opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let card = match results.first().and_then(|r| r.cards().first()) {
        Some(c) => *c,
        None => return Ok(()),
    };
    // Using the effect of a Supporter is not playing it: bypass `supporterTurn`.
    let supporter_turn = g.st.players[p].supporter_turn;
    g.st.players[p].supporter_turn = 0;
    let r = g.run_fx(Effect::Trainer { p: p as u8, card, target: None, via_attack: true });
    g.st.players[p].supporter_turn = supporter_turn;
    match r {
        Ok(_) => Ok(()),
        // `catch (error)`: a GameError (not a TypeError) means this Supporter's
        // effect can't be used right now: choose another one.
        Err(e) if !e.0.starts_with("TypeError") => {
            let opp = 1 - p;
            let mut blocked = Blocked(f.a[1] as u32 as u64 | ((f.a[2] as u32 as u64) << 32));
            let index = g.st.players[opp].hand.iter().position(|c| c == card).unwrap_or(0) as u8;
            blocked.push(index);
            choose_supporter(g, me, p, opp, blocked)
        }
        Err(e) => Err(e),
    }
}
