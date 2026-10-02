//! Mega Gardevoir ex (M1S / ASC): Overflowing Wishes - for each of your
//! Benched Pokémon, search your deck for a Basic [P] Energy and attach it to
//! that Pokémon, then shuffle. Mega Symphonia - 50 damage for each [P] Energy
//! attached to all of your Pokémon.
//!
//! Twinleaf: Overflowing Wishes does nothing (no shuffle) without a Benched
//! Pokémon or with an empty deck; otherwise an AttachEnergyPrompt over the
//! deck (Bench only, "Psychic Energy", differentTargets, no cancel); each
//! transfer is a MOVE_CARDS (no AttachEnergyEffect), then SHUFFLE_DECK (also
//! for an empty answer). Mega Symphonia sums, over every Pokémon's
//! CheckProvidedEnergyEffect, the provided [P] and [ANY] entries.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaGardevoirex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            return Ok(());
        }
        if pl.deck.is_empty() {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut o = AttachOpts::new(pl.deck.len().min(255) as u8);
        o.allow_cancel = false;
        o.different_targets = true;
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy {
                cards: ListRef::Deck(p as u8),
                player_type: PlayerType::BottomPlayer,
                slots,
                filter: Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Psychic Energy"), ..Filter::none() },
                o,
            },
            Cont::Card { card: me, frame: f },
        );
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut energies = 0;
        for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, s), energy_map: SVec::new() })?;
            if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                for m in energy_map.iter() {
                    energies += m.provides.iter().filter(|t| **t == ct::PSYCHIC || **t == ct::ANY).count() as i32;
                }
            }
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = energies * 50;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    if transfers.is_empty() {
        shuffle_deck(g, p);
        return Ok(());
    }
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Deck(p as u8), ListRef::Slot(target.p, target.s), &[c], me)?;
    }
    shuffle_deck(g, p);
    Ok(())
}
