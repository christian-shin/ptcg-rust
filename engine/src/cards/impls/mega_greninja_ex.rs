//! Mega Greninja ex (CRI / M4): Mortal Shuriken — once during your turn, if
//! this Pokémon is Active, you may discard a Basic [W] Energy card from your
//! hand; place 6 damage counters on 1 of your opponent's Pokémon. Ninja
//! Spinner — 120; you may put a [W] Energy attached to this Pokémon into your
//! hand for 80 more damage.
//!
//! Twinleaf: the Ability throws unless this card is the Active, the player
//! marker is unset, the hand holds a Basic Energy that provides [W] and the
//! opponent has a Pokémon. The discard prompt filters by the name "Water
//! Energy" (min 1, max 1, cancel allowed); a cancel does nothing at all. The
//! marker and board effect are only set after the target is chosen (even if
//! no target comes back). The marker is also cleared when this card is
//! played. Ninja Spinner: Confirm, then CheckProvidedEnergy + ChooseEnergy
//! ([W], no cancel) and a CardsToHandEffect on the Active; the +80 is added
//! whenever the prompt returned at least one card.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "MegaGreninjaex",
    mask: mask(&[k::POWER, k::END_TURN, k::PLAY_POKEMON, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn shuriken() -> crate::markers::MarkerName {
    crate::marker!("MORTAL_SHURIKEN_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.active_pokemon(p) != Some(me) {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(shuriken(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let has_w = g.st.players[p].hand.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::WATER)
        });
        if !has_w {
            bail!("CANNOT_USE_POWER");
        }
        let o = 1 - p;
        let has_opp = {
            let pl = &g.st.players[o];
            !pl.slots[pl.active as usize].cards.is_empty() || pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty())
        };
        if !has_opp {
            bail!("CANNOT_USE_POWER");
        }
        let filter = Filter {
            super_type: Some(SuperType::Energy as u8),
            energy_type: Some(EnergyType::Basic as u8),
            name: Some("Water Energy"),
            ..Filter::none()
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), filter, ChooseCardsOpts::new(1, 1, true), Cont::Card { card: me, frame: f });
        return Ok(());
    }

    remove_marker_at_end_of_turn(g, e, shuriken(), me);
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(shuriken(), me);
        }
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(3);
        f.a[0] = p as i32;
        f.e[0] = e;
        confirmation_prompt(g, p, "WANT_TO_USE_EFFECT_OF_ATTACK", Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if cards.is_empty() {
                return Ok(());
            }
            move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_DAMAGE",
                PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let targets: Vec<SlotRef> = first.slots().to_vec();
            if let Some(t) = targets.first() {
                g.run_fx(Effect::PlaceDamageCounters { p: p as u8, target: *t, damage: 60, source: me })?;
            }
            g.st.players[p].marker.add(shuriken(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            ability_used(g, p, me);
            Ok(())
        }
        3 => {
            let atk = f.e[0];
            if !first.as_bool() {
                g.release_fx(atk);
                return Ok(());
            }
            let r = (|| -> R {
                let a = g.st.players[p].active;
                let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
                let energy = match pe {
                    Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
                    _ => SVec::new(),
                };
                let mut cost = SVec::new();
                cost.push(ct::WATER);
                let mut nf = CardFrame::at(4);
                nf.a[0] = p as i32;
                nf.e[0] = atk;
                let id = g.player_id(p);
                g.prompt(id, "CHOOSE_CARD_TO_HAND", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, Cont::Card { card: me, frame: nf });
                Ok(())
            })();
            if r.is_err() {
                g.release_fx(atk);
            }
            r
        }
        4 => {
            let atk = f.e[0];
            let r = (|| -> R {
                let cards: Vec<CardId> = match first {
                    Res::Energy(c) => c.as_slice().to_vec(),
                    _ => Vec::new(),
                };
                if !cards.is_empty() {
                    let (opp, attack, source) = match *g.e(atk) {
                        Effect::Attack { opp, attack, source, .. } => (opp, attack, source),
                        _ => return Ok(()),
                    };
                    let a = g.st.players[p].active;
                    let mut list = SVec::new();
                    for c in cards {
                        list.push(c);
                    }
                    let b = AtkBase { attack_effect: atk, player: p as u8, opponent: opp, attack, source, target: SlotRef::new(p, a) };
                    g.run_fx(Effect::CardsToHand { b, cards: list })?;
                    if let Effect::Attack { damage, .. } = g.e_mut(atk) {
                        *damage += 80;
                    }
                }
                Ok(())
            })();
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}
