//! Raging Bolt ex (TEF): Burst Roar — discard your hand and draw 6 cards.
//! Bellowing Thunder — you may discard any amount of Basic Energy from your
//! Pokémon; 70 damage for each card discarded.
//!
//! Twinleaf: Burst Roar throws with an empty deck; the hand goes to the
//! discard in one MOVE_CARDS, then MOVE_CARDS { count: 6 } deck→hand.
//! Bellowing Thunder zeroes the damage, then
//! DISCARD_UP_TO_X_ENERGY_FROM_YOUR_POKEMON (max = available Basic Energy,
//! min 0, no cancel); one DiscardCardsEffect per source slot (first-seen
//! order), then `damage = transfers.length * 70`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "RagingBoltex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn basic_energy() -> Filter {
    Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p,
            _ => return Ok(()),
        };
        if g.st.players[p as usize].deck.is_empty() {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        g.run_fx(Effect::MoveCards {
            source: ListRef::Hand(p),
            destination: ListRef::Discard(p),
            cards: None,
            count: None,
            to_top: false,
            to_bottom: false,
            skip_cleanup: false,
            source_card: me,
        })?;
        move_count_from(g, ListRef::Deck(p), ListRef::Hand(p), 6, me)?;
    }

    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 0;
        }
        let filter = basic_energy();
        let mut available = 0usize;
        for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            available += g.st.slot(p, s).cards.iter().filter(|c| {
                let d = g.st.cdef(*c);
                d.is_energy() && d.energy_type == EnergyType::Basic as u8
            }).count();
        }
        if available == 0 {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let o = MoveOpts { allow_cancel: false, min: 0, max: Some(available.min(255) as u8), ..Default::default() };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_ENERGIES_TO_DISCARD",
            PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let e = f.e[0];
    let r = (|| -> R {
        let transfers = match results.first().copied() {
            Some(Res::CardsFrom(t)) => t,
            _ => return Ok(()),
        };
        if transfers.is_empty() {
            return Ok(());
        }
        let mut groups: Vec<(SlotRef, SVec<CardId, 16>)> = Vec::new();
        for (from, c) in transfers.iter() {
            let s = get_target(&g.st, p, *from)?;
            match groups.iter_mut().find(|(t, _)| *t == s) {
                Some((_, v)) => v.push(*c),
                None => {
                    let mut v = SVec::new();
                    v.push(*c);
                    groups.push((s, v));
                }
            }
        }
        let (opp, attack, source) = match *g.e(e) {
            Effect::Attack { opp, attack, source, .. } => (opp, attack, source),
            _ => return Ok(()),
        };
        for (target, cards) in groups {
            let b = AtkBase { attack_effect: e, player: p as u8, opponent: opp, attack, source, target };
            g.run_fx(Effect::DiscardCards { b, cards })?;
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = transfers.len() as i32 * 70;
        }
        Ok(())
    })();
    g.release_fx(e);
    r
}
