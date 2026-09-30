//! Greninja ex (TWM, Tera): Shinobi Blade — 170; you may search your deck
//! for a card and put it into your hand, then shuffle. Mirage Barrage —
//! discard 2 Energy from this Pokémon; 120 damage to 2 of your opponent's
//! Pokémon.
//!
//! Twinleaf: Shinobi Blade throws CANNOT_USE_POWER on an empty deck, then a
//! Confirm (SEARCH_DECK_FOR_CARD); yes → ChooseCardsPrompt (min 1, max 1, no
//! filter) → MOVE_CARDS deck→hand → bare ShuffleDeckPrompt. Mirage Barrage:
//! ChooseEnergyPrompt ([C][C] over the Active's energy map) → ChoosePokemon
//! (opponent, Active/Bench, min 1, max 2) → DAMAGE_OPPONENT_POKEMON(120) and
//! only then the DiscardCardsEffect of the chosen energy (target Active).
//! The Tera rule prevents PutDamageEffects on this Pokémon on the Bench.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Greninjaex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        confirmation_prompt(g, p, "SEARCH_DECK_FOR_CARD", Cont::Card { card: me, frame: f });
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
        let energy = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
            _ => SVec::new(),
        };
        let mut cost = SVec::new();
        cost.push(ct::COLORLESS);
        cost.push(ct::COLORLESS);
        g.retain_fx(e);
        let mut f = CardFrame::at(3);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, Cont::Card { card: me, frame: f });
    }
    tera_rule(g, e, me);
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if !first.as_bool() {
                return Ok(());
            }
            let mut nf = f;
            nf.stage = 2;
            choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
            Ok(())
        }
        3 => {
            // Energy chosen (at most 2 cards for [C][C]): keep them for after the damage.
            let mut nf = f;
            nf.stage = 4;
            nf.a[1] = -1;
            nf.a[2] = -1;
            if let Res::Energy(c) = first {
                for (i, x) in c.iter().enumerate().take(2) {
                    nf.a[1 + i] = *x as i32;
                }
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_DAMAGE",
                PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 2, allow_cancel: false, blocked: SVec::new() },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        4 => {
            let atk = f.e[0];
            let r = (|| -> R {
                let targets: Vec<SlotRef> = first.slots().to_vec();
                damage_opponent_pokemon(g, atk, 120, &targets)?;
                let mut cards = SVec::new();
                for &c in &f.a[1..3] {
                    if c >= 0 {
                        cards.push(c as CardId);
                    }
                }
                if let Effect::Attack { p: ap, opp, attack, source, .. } = *g.e(atk) {
                    let pp = ap as usize;
                    let target = SlotRef::new(pp, g.st.players[pp].active);
                    g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: atk, player: ap, opponent: opp, attack, source, target }, cards })?;
                }
                Ok(())
            })();
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}
