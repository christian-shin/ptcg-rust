//! Hydrapple (DRI 18): Hydra Breath — discard 6 Basic [G] Energy from your
//! hand in order to Knock Out your opponent's Active Pokémon. Whip Smash — 140.
//!
//! Twinleaf: counts Basic energy cards named 'Grass Energy' in hand; with 6+
//! a non-cancellable ChooseCardsPrompt (exactly 6) from hand, then
//! MOVE_CARDS to the discard and a KnockOutOpponentEffect on the opponent's
//! Active.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Hydrapple", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !after_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let e = real_attack(g, e);
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let n = g.st.players[p]
        .hand
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.name == "Grass Energy"
        })
        .count();
    if n < 6 {
        return Ok(());
    }
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    f.a[0] = p as i32;
    let filter = Filter {
        super_type: Some(SuperType::Energy as u8),
        energy_type: Some(EnergyType::Basic as u8),
        name: Some("Grass Energy"),
        ..Filter::none()
    };
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), filter, ChooseCardsOpts::new(6, 6, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let r = (|| {
        let p = f.a[0] as usize;
        let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
        if cards.is_empty() {
            return Ok(());
        }
        move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, NO_CARD)?;
        let (pp, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let o = opp as usize;
        let a = g.st.players[o].active;
        let b = AtkBase { attack_effect: atk, player: pp, opponent: opp, attack, source, target: SlotRef::new(o, a) };
        g.run_fx(Effect::KnockOutOpponent { b, knocked_out: false, prize_count: 0 })?;
        Ok(())
    })();
    g.release_fx(atk);
    r
}
