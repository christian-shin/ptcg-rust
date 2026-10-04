//! N's Darmanitan (JTG): Backdraft — 30 damage for each Basic Energy card in
//! your opponent's discard pile. Darman-i-cannon — 90; discard all Energy
//! from this Pokémon; also 90 damage to 1 of your opponent's Benched Pokémon.
//!
//! Twinleaf: Backdraft assigns the damage. Darman-i-cannon discards every
//! card of the Active's CheckProvidedEnergy map with one DiscardCardsEffect,
//! then opens a non-cancellable ChoosePokemonPrompt over the opponent's Bench
//! and puts 90 with a plain PutDamageEffect on the chosen Pokémon.
//!
//! Fixed (phase 4b, W4): with an empty Bench the prompt (min 1) had no valid
//! answer; the attack now stops after the discard when there is no Benched
//! Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NsDarmanitan", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let n = g.st.players[opp].discard.iter().filter(|c| {
            let d = g.st.cdef(*c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        }).count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = n * 30;
        }
    }

    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let pu = p as usize;
        let a = g.st.players[pu].active;
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: SlotRef::new(pu, a), energy_map: SVec::new() })?;
        let mut cards: SVec<CardId, 16> = SVec::new();
        if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
            for en in energy_map.iter() {
                cards.push(en.card);
            }
        }
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(pu, a) };
        g.run_fx(Effect::DiscardCards { b, cards })?;

        let benched = g.st.players[opp as usize].bench.iter().filter(|s| !g.st.players[opp as usize].slots[**s as usize].cards.is_empty()).count();
        if benched == 0 {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(pu);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let r = (|| -> R {
        for t in targets {
            put_damage(g, atk, 90, t)?;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
