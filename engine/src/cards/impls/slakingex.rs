//! Slaking ex (SSP): Born to Slack — if your opponent has no Pokémon ex or
//! Pokémon V in play, this Pokémon can't attack. Great Swing — 280; discard
//! an Energy from this Pokémon.
//!
//! Twinleaf: any AttackEffect while this card is the attacker's Active
//! Pokémon throws BLOCKED_BY_ABILITY unless the opponent has an ex / V /
//! VMAX / VSTAR / V-UNION in play or the ability is blocked. Great Swing
//! prices the discard as [C] with a non-cancellable ChooseEnergyPrompt and
//! reduces a DiscardCardsEffect on `player.active`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Slakingex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::Attack { p, .. } = *g.e(e) {
        let p = p as usize;
        if g.st.active_pokemon(p) == Some(me) {
            let o = 1 - p;
            let special = for_each_pokemon(g, o, PlayerType::TopPlayer).iter().any(|x| {
                let d = g.st.cdef(x.1);
                d.has_tag(tag::POKEMON_EX_LOWER)
                    || d.has_tag(tag::POKEMON_V)
                    || d.has_tag(tag::POKEMON_VMAX)
                    || d.has_tag(tag::POKEMON_VSTAR)
                    || d.has_tag(tag::POKEMON_VUNION)
            });
            if !is_ability_blocked(g, p, me, None) && !special {
                bail!("BLOCKED_BY_ABILITY");
            }
        }
    }
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let (sp, ss) = match g.st.find_pokemon_slot(me) {
            Some(x) => x,
            None => bail!("TypeError: findCardList"),
        };
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(sp, ss), energy_map: SVec::new() })?;
        let energy = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
            _ => SVec::new(),
        };
        let mut cost = SVec::new();
        cost.push(ct::COLORLESS);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let cards: SVec<CardId, 64> = match results.first() {
        Some(Res::Energy(c)) => {
            let mut v = SVec::new();
            for x in c.as_slice() {
                v.push(*x);
            }
            v
        }
        _ => SVec::new(),
    };
    let r = (|| -> R {
        let (p, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let a = g.st.players[p as usize].active;
        let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: SlotRef::new(p as usize, a) };
        g.run_fx(Effect::DiscardCards { b, cards })?;
        Ok(())
    })();
    g.release_fx(atk);
    r
}
