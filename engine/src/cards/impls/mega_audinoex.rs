//! Mega Audino ex (MC): Kaleidowaltz — flip 3 coins; for each heads, search
//! your deck for up to 2 Basic Energy cards and attach them to your Pokémon
//! in any way you like, then shuffle. Ear Force — 20+, 80 more for each
//! Energy attached to your opponent's Active Pokémon.
//!
//! Twinleaf: Ear Force counts the `provides` entries of the opponent's
//! CheckProvidedEnergyEffect (Active by default).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaAudinoex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        coin_flip_sequence(g, p, 3, CoinCb::SequenceCard { card: me, frame: f })?;
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp,
            _ => return Ok(()),
        };
        let o = opp as usize;
        let src = SlotRef::new(o, g.st.players[o].active);
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: opp, source: src, energy_map: SVec::new() })?;
        let n: i32 = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map.iter().map(|m| m.provides.len() as i32).sum(),
            _ => 0,
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += n * 80;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let heads = (f.a[2] as u32).count_ones() as u8;
            let n = heads * 2;
            if n == 0 {
                shuffle_deck(g, p);
                return Ok(());
            }
            let mut o = AttachOpts::new(g.st.players[p].deck.len() as u8);
            o.allow_cancel = false;
            o.min = 0;
            o.max = n;
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            let mut filter = Filter::super_type(SuperType::Energy);
            filter.energy_type = Some(EnergyType::Basic as u8);
            let nf = CardFrame { stage: 2, ..f };
            let id = g.player_id(p);
            g.prompt(
                id,
                "ATTACH_ENERGY_CARDS",
                PromptKind::AttachEnergy { cards: ListRef::Deck(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let transfers: SVec<(CardTarget, CardId), 64> = match results.first() {
                Some(Res::Attach(t)) => *t,
                _ => SVec::new(),
            };
            for (to, c) in transfers.iter().copied() {
                let target = get_target(&g.st, p, to)?;
                move_cards(g, ListRef::Deck(p as u8), target.list(), &[c], me)?;
            }
            shuffle_deck(g, p);
            Ok(())
        }
        _ => Ok(()),
    }
}
