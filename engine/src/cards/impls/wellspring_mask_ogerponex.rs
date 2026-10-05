//! Wellspring Mask Ogerpon ex (TWM, Tera): Sob - 20; the Defending Pokémon
//! can't retreat during your opponent's next turn. Torrential Pump - 100;
//! you may shuffle 3 Energy from this Pokémon into your deck; if you do,
//! also 120 damage to 1 of your opponent's Benched Pokémon. Tera: no attack
//! damage while on the Bench.
//!
//! Twinleaf quirks kept: declining sets the attack's damage back to exactly
//! 100 (dropping any bonus added during the AttackEffect), and the energy
//! choice is priced as [C][C][C]. Twinleaf builds the energy map before the
//! confirm prompt; nothing can change the Active's energy in between, so it
//! is rebuilt when the confirm resolves (the frame can't hold the map).
//! R7A (ruling 1580): the Energy goes back into the deck, and the deck is shuffled, after the damage (`move_cards_after_damage`,
//! `shuffle_deck_after_damage`); the bench target is asked right after the Energy choice.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "WellspringMaskOgerponex",
    mask: mask(&[k::ATTACK, k::PUT_DAMAGE]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        return block_retreat(g, e);
    }
    if was_attack_used(g, e, 1, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let op = &g.st.players[o];
        if !op.bench.iter().any(|b| !op.slots[*b as usize].cards.is_empty()) {
            return Ok(());
        }
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(id, "WANT_TO_USE_ABILITY", PromptKind::Confirm, Cont::Card { card: me, frame: f });
    }
    tera_rule(g, e, me);
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let id = g.player_id(p);
    let r = (|| -> R<bool> {
        match f.stage {
            1 => {
                if !first.as_bool() {
                    if let Effect::Attack { damage, .. } = g.e_mut(atk) {
                        *damage = 100;
                    }
                    return Ok(true);
                }
                let a = g.st.players[p].active;
                let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
                let energy = match pe {
                    Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
                    _ => SVec::new(),
                };
                let mut cost = SVec::new();
                for _ in 0..3 {
                    cost.push(ct::COLORLESS);
                }
                let mut nf = f;
                nf.stage = 2;
                g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, Cont::Card { card: me, frame: nf });
                Ok(false)
            }
            2 => {
                let cards: Vec<CardId> = match first {
                    Res::Energy(c) => c.as_slice().to_vec(),
                    _ => Vec::new(),
                };
                let a = g.st.players[p].active;
                // The Energy goes back into the deck, and the deck is shuffled, after the damage.
                move_cards_after_damage(g, atk, ListRef::Slot(p as u8, a), ListRef::Deck(p as u8), &cards, me)?;
                shuffle_deck_after_damage(g, atk, p);
                let mut slots = SVec::new();
                slots.push(SlotType::Bench as u8);
                let mut nf = f;
                nf.stage = 4;
                g.prompt(
                    id,
                    "CHOOSE_POKEMON_TO_DAMAGE",
                    PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
                    Cont::Card { card: me, frame: nf },
                );
                Ok(false)
            }
            4 => {
                if let Some(t) = first.slots().first().copied() {
                    put_damage(g, atk, 120, t)?;
                }
                Ok(true)
            }
            _ => Ok(true),
        }
    })();
    match r {
        Ok(true) | Err(_) => g.release_fx(atk),
        Ok(false) => {}
    }
    r.map(|_| ())
}
