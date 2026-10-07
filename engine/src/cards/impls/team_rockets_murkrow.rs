//! Team Rocket's Murkrow (DRI): Deceit — search your deck for a Supporter,
//! reveal it, put it into your hand, then shuffle. Torment — 30; choose 1 of
//! the opponent's Active Pokémon's attacks; it can't be used next turn.
//!
//! Twinleaf: Deceit does nothing with an empty deck; the search can be
//! cancelled (no reveal); the shuffle is created in the ShowCards callback
//! (fixed in R1-3: directly in the search callback when nothing was taken,
//! so the deck is always shuffled) and has no animation wait. Torment is
//! OPPONENTS_POKEMON_CANNOT_USE_THAT_ATTACK (printed attacks of the current
//! Active; an OpponentPokemonCannotUseAttackEffect when answered).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsMurkrow", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let mut opts = ChooseCardsOpts::new(0, 1, true);
        for (i, c) in g.st.players[p].deck.iter().enumerate() {
            let d = g.st.cdef(c);
            if !(d.is_trainer() && d.trainer_type == TrainerType::Supporter as u8) {
                opts.blocked.push(i as u8);
            }
        }
        let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Supporter as u8), ..Default::default() };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, opts, Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if after_attack_used(g, e, 1, me) {
        let e = real_attack(g, e);
        opponents_pokemon_cannot_use_that_attack(g, me, e);
    }
    Ok(())
}

/// `OPPONENTS_POKEMON_CANNOT_USE_THAT_ATTACK(store, state, effect, source)`.
fn opponents_pokemon_cannot_use_that_attack(g: &mut Game, me: CardId, e: EffId) {
    let (p, o) = match *g.e(e) {
        Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
        _ => return,
    };
    let pc = match g.st.active_pokemon(o) {
        Some(c) => c,
        None => return,
    };
    if g.st.cdef(pc).attacks.is_empty() {
        return;
    }
    g.retain_fx(e);
    let mut cards = SVec::new();
    cards.push(pc);
    let mut f = CardFrame::at(3);
    f.a[0] = p as i32;
    f.e[0] = e;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_ATTACK_TO_DISABLE",
        PromptKind::ChooseAttack { cards, allow_cancel: false, blocked_message: "NOT_ENOUGH_ENERGY", blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            if cards.is_empty() {
                // Nothing found: the deck is still shuffled.
                let id = g.player_id(p);
                g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
                return Ok(());
            }
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let oid = g.player_id(1 - p);
            g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
            Ok(())
        }
        3 => {
            let atk = f.e[0];
            let r = disable(g, atk, results);
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}

fn disable(g: &mut Game, atk: EffId, results: &[Res]) -> R {
    let a = match results.first() {
        Some(Res::Attack(a)) => *a,
        _ => return Ok(()),
    };
    let name = g.st.cdef(a.card).attacks[a.index as usize].tl_name;
    let b = match *g.e(atk) {
        Effect::Attack { p, opp, attack, source, .. } => {
            let o = opp as usize;
            AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: SlotRef::new(o, g.st.players[o].active) }
        }
        _ => return Ok(()),
    };
    g.run_fx(Effect::OpponentPokemonCannotUseAttack { b, name })?;
    Ok(())
}
