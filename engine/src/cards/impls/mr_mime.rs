//! Mr. Mime (TEF): Look-Alike Show — your opponent reveals their hand; you may
//! use the effect of a Supporter card you find there as the effect of this
//! attack. Eerie Wave — 20; your opponent's Active Pokémon is now Confused.
//!
//! Twinleaf: Look-Alike Show throws CANNOT_USE_POWER when this Pokémon has a
//! Special Condition or is not the Active, then opens a ChooseCardsPrompt on
//! the opponent's hand (Supporters, 0..1, no cancel); the callback reduces a
//! TrainerEffect for the chosen card with the attacker as player.
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
                if !g.st.slot(q as usize, s).special_conditions.is_empty() {
                    bail!("CANNOT_USE_POWER");
                }
                if !(q as usize == p && s == g.st.players[p].active) {
                    bail!("CANNOT_USE_POWER");
                }
            }
            _ => bail!("CANNOT_USE_POWER"),
        }
        let mut filter = Filter::super_type(SuperType::Trainer);
        filter.trainer_type = Some(TrainerType::Supporter as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_COPY_EFFECT", ListRef::Hand(opp as u8), filter, ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }

    if was_attack_used(g, e, 1, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Confused])?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0];
    let card = match results.first().and_then(|r| r.cards().first()) {
        Some(c) => *c,
        None => return Ok(()),
    };
    g.run_fx(Effect::Trainer { p: p as u8, card, target: None })?;
    Ok(())
}
