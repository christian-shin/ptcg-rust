//! Team Rocket's Honchkrow (M2a): Rocket Feathers — discard any number of
//! "Team Rocket" Supporters from your hand; 60 damage for each. Hammer In —
//! 100.
//!
//! Twinleaf: the hand prompt (Trainer filter, non-Team Rocket Supporters
//! blocked, max = their count) is shown even when there are none; choosing
//! nothing sets the damage to 0.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsHonchkrow", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn is_rocket_supporter(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    d.is_trainer() && d.trainer_type == TrainerType::Supporter as u8 && d.name.contains("Team Rocket")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let mut n = 0u8;
    let mut opts = ChooseCardsOpts::new(0, 0, false);
    for (i, c) in g.st.players[p].hand.iter().enumerate() {
        if is_rocket_supporter(g, c) {
            n += 1;
        } else {
            opts.blocked.push(i as u8);
        }
    }
    opts.max = n;
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.e[0] = e;
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), Filter::super_type(SuperType::Trainer), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    let r = if cards.is_empty() {
        Ok(())
    } else {
        move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)
    };
    if r.is_ok() {
        let n = cards.len() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage = 60 * n;
        }
    }
    g.release_fx(atk);
    r
}
