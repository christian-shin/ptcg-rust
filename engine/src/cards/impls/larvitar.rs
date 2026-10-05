//! Larvitar (JTG): Crunch — 20; flip a coin. If heads, discard an Energy
//! from your opponent's Active Pokémon. Ported so Tyranitar (JTG) can evolve
//! in check decks.
//!
//! Twinleaf: on heads with an Energy card in the Defending Pokémon's list, a
//! ChooseCardsPrompt (Energy, exactly 1) on that list; the callback reduces a
//! DiscardCardsEffect.
//!
//! Fixed (phase 4b, R2): the prompt could be cancelled, so the discard that
//! the card text makes mandatory could be skipped. The coin flip and the
//! discard came before the damage (the Defending Pokémon's Spiky Energy was
//! already gone); they now run in AfterAttackEffect (a fresh AttackEffect's
//! data).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Larvitar", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !after_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::AfterAttack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    if let Err(err) = g.coin_flip(p, CoinCb::Card { card: me, frame: f }) {
        g.release_fx(e);
        return Err(err);
    }
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    let atk = f.e[0];
    if !heads {
        g.release_fx(atk);
        return Ok(());
    }
    let (p, o) = match *g.e(atk) {
        Effect::AfterAttack { p, opp, .. } => (p as usize, opp as usize),
        _ => {
            g.release_fx(atk);
            return Ok(());
        }
    };
    let a = g.st.players[o].active;
    if !g.st.slot(o, a).cards.iter().any(|c| g.st.cdef(c).is_energy()) {
        g.release_fx(atk);
        return Ok(());
    }
    let mut nf = CardFrame::at(2);
    nf.e[0] = atk;
    choose_cards(
        g,
        p,
        "CHOOSE_CARD_TO_DISCARD",
        ListRef::Slot(o as u8, a),
        Filter::super_type(SuperType::Energy),
        ChooseCardsOpts::new(1, 1, false),
        Cont::Card { card: me, frame: nf },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 2 {
        return Ok(());
    }
    let atk = f.e[0];
    let mut cards = SVec::new();
    if let Some(r) = results.first() {
        for c in r.cards() {
            cards.push(*c);
        }
    }
    let r = (|| -> R {
        let (p, opp, attack, source) = match attack_data(g, atk) {
            Some(d) => d,
            None => return Ok(()),
        };
        let a = g.st.players[opp as usize].active;
        let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: SlotRef::new(opp as usize, a) };
        g.run_fx(Effect::DiscardCards { b, cards })?;
        Ok(())
    })();
    g.release_fx(atk);
    r
}
