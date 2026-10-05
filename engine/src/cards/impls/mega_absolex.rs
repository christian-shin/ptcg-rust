//! Mega Absol ex (M1L / MEG 86): Terminal Period — if the opponent's Active
//! has exactly 6 damage counters, it is Knocked Out. Claw of Darkness — 200,
//! the opponent reveals their hand and you discard a card from it.
//!
//! Twinleaf: Terminal Period is a KnockOutOpponentEffect (fixed in phase 4b,
//! R4: it added 999 damage straight onto the Active, which Mist Energy and
//! effect prevention could not stop); the hand prompt is a plain
//! ChooseCardsPrompt on the opponent's hand (no ShowCards).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaAbsolex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, opp, attack, source, .. } = *g.e(e) {
            let o = opp as usize;
            let a = g.st.players[o].active;
            if g.st.slot(o, a).damage == 60 {
                let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(o, a) };
                g.run_fx(Effect::KnockOutOpponent { b, knocked_out: false, prize_count: 0 })?;
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        if g.st.players[o].hand.is_empty() {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = o as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(o as u8), Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let o = f.a[0] as usize;
    let sel: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if sel.is_empty() {
        return Ok(());
    }
    move_cards(g, ListRef::Hand(o as u8), ListRef::Discard(o as u8), &[sel[0]], me)
}
