//! Goldeen (TWM): Festival Lead. Whirlpool — 10; flip a coin, if heads
//! discard an Energy from the opponent's Active Pokémon.
//!
//! Twinleaf: Festival Lead's runtime `barrage` flag is written on every use
//! (fixed in phase 4b, R4: it used to be skipped when the opponent's Active had
//! no Energy card, and left unchanged while the Ability was blocked), before
//! the no-Energy return. The coin flip is still skipped without Energy.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Goldeen@Goldeen TWM|Goldeen PRE",
    mask: mask(&[k::ATTACK, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: Some(coin),
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        return after_attack(g, me, e);
    }
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    // Festival Lead: `barrage = Ability works && Festival Grounds in play`, written on every use.
    let blocked = is_ability_blocked(g, p, me, None);
    let fg = g.st.stadium_card().map(|s| g.st.cdef(s).name == "Festival Grounds").unwrap_or(false);
    crate::copy_attack::write_barrage(g, me, |b, shown| {
        if !blocked && fg {
            *b |= 1;
        } else {
            *b &= !1;
        }
        *shown |= 1;
    });
    Ok(())
}

/// The coin, the Energy check and the discard are asked after the damage (user rule 2026-10-07).
fn after_attack(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, opp) = match *g.e(e) {
        Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
        _ => return Ok(()),
    };
    let oa = g.st.players[opp].active;
    if !g.st.slot(opp, oa).cards.iter().any(|c| g.st.cdef(c).is_energy()) {
        return Ok(());
    }
    g.retain_fx(e);
    let mut f = CardFrame::at(0);
    f.a[0] = p as i32;
    f.e[0] = e;
    g.coin_flip(p, CoinCb::Card { card: me, frame: f })?;
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    if !heads {
        g.release_fx(f.e[0]);
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    let oa = g.st.players[o].active;
    let mut nf = f;
    nf.stage = 1;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_CARD_TO_DISCARD",
        PromptKind::ChooseCards { cards: ListRef::Slot(o as u8, oa), filter: Filter::super_type(SuperType::Energy), opts: ChooseCardsOpts::new(1, 1, false) },
        Cont::Card { card: me, frame: nf },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let first = results.first().copied().unwrap_or(Res::Null);
    let r = (|| -> R {
        let card = match first.cards().first() {
            Some(c) => *c,
            None => bail!("TypeError: selected[0]"),
        };
        let (p, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let target = SlotRef::new(opp as usize, g.st.players[opp as usize].active);
        let mut cards = SVec::new();
        cards.push(card);
        g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target }, cards })?;
        Ok(())
    })();
    g.release_fx(atk);
    r
}
