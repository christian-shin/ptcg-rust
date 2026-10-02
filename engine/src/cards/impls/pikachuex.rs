//! Pikachu ex (SSP, Tera): Tenacious Heart — if this Pokémon has full HP and
//! would be Knocked Out by an attack, it isn't Knocked Out and its remaining
//! HP becomes 10 instead. Topaz Bolt — 300; discard 3 Energy from this
//! Pokémon. Tera: no attack damage while on the Bench.
//!
//! Twinleaf: Tenacious Heart is the shared SURVIVE_ON_TEN_IF_FULL_HP logic
//! (see Crustle BCR): any PutDamageEffect on a slot holding this card, unless
//! the ability is blocked for the owner, with the slot undamaged and the
//! damage >= its HP. Topaz Bolt prices the discard as [C][C][C] with a
//! non-cancellable ChooseEnergyPrompt over the Active's CheckProvidedEnergy
//! map and reduces a DiscardCardsEffect on `player.active`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Pikachuex@SSP|ASC", mask: mask(&[k::PUT_DAMAGE, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    super::crustle_bcr::survive_on_ten_if_full_hp(g, me, e)?;

    if was_attack_used(g, e, 0, me) {
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
        cost.push(ct::COLORLESS);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, Cont::Card { card: me, frame: f });
    }

    tera_rule(g, e, me);
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let cards: SVec<CardId, 16> = match results.first() {
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
