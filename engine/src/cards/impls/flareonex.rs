//! Flareon ex (PRE, Tera): Burning Charge — 130; search your deck for up to 2
//! Basic Energy and attach them to 1 of your Pokémon, then shuffle. Carnelian
//! — 280; during your next turn, this Pokémon can't attack. Tera: no damage
//! from attacks while on the Bench.
//!
//! Twinleaf: Burning Charge does nothing with an empty deck or no Benched
//! Pokémon. A 0-2 energy choice (no cancel) is followed (only if any were
//! chosen) by a ChoosePokemonPrompt; the cards move deck -> that Pokémon
//! (no reveal). The deck is shuffled afterwards (not reached if the target
//! prompt returned no target).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Flareonex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let pl = &g.st.players[p];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        g.retain_fx(e);
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 2, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }

    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }

    if let Effect::PutDamage { b, .. } = *g.e(e) {
        let t = b.target;
        let slot = g.st.slot(t.p as usize, t.s);
        if slot.cards.contains(me) && g.st.slot_pokemon(t.p as usize, t.s) == Some(me) {
            let pl = b.player as usize;
            let op = 1 - pl;
            if (t.p as usize == pl && t.s == g.st.players[pl].active) || (t.p as usize == op && t.s == g.st.players[op].active) {
                return Ok(());
            }
            g.set_prevent(e, true);
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        // Energy chosen.
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if cards.is_empty() {
                g.release_fx(atk);
                shuffle_deck(g, p);
                return Ok(());
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.e[0] = atk;
            for (i, c) in cards.iter().enumerate() {
                nf.l[i] = *c;
            }
            nf.a[1] = cards.len() as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_ATTACH_CARDS",
                PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        // Target chosen.
        2 => {
            let r = (|| -> R {
                let t = match first.slots().first() {
                    Some(t) => *t,
                    None => return Ok(()),
                };
                let n = f.a[1] as usize;
                let cards: Vec<CardId> = f.l[..n].to_vec();
                let source = match *g.e(atk) {
                    Effect::Attack { source, .. } => source,
                    _ => return Ok(()),
                };
                let src_card = g.st.slot_pokemon(source.p as usize, source.s).unwrap_or(me);
                move_cards(g, ListRef::Deck(p as u8), ListRef::Slot(t.p, t.s), &cards, src_card)?;
                shuffle_deck(g, p);
                Ok(())
            })();
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}
