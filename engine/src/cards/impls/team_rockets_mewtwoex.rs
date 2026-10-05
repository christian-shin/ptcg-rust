//! Team Rocket's Mewtwo ex (DRI): Power Saver — can't attack unless you have
//! 4 or more Team Rocket's Pokémon in play. Erasure Ball — 160+; discard up
//! to 2 Energy from your Benched Pokémon, 60 more damage for each.
//!
//! Twinleaf: Power Saver reacts to any UseAttackEffect whose source slot
//! holds this card (after the Ability-lock check). Erasure Ball's prompt
//! allows any Energy (phase 4b: the filter used to be Basic only, but the text
//! says "Energy"), is skipped without a Benched Pokémon, and each chosen card
//! is a separate MOVE_CARDS to the discard pile.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsMewtwoex", mask: mask(&[k::USE_ATTACK, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::UseAttack { p, source, .. } = *g.e(e) {
        if g.st.slot(source.p as usize, source.s).cards.contains(me) {
            let p = p as usize;
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            let n = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().filter(|(_, c, _)| g.st.cdef(*c).has_tag(tag::TEAM_ROCKET)).count();
            if n < 4 {
                bail!("CANNOT_USE_ATTACK");
            }
        }
    }
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        let has_bench = pl.bench.iter().any(|s| !pl.slots[*s as usize].cards.is_empty());
        if !has_bench {
            return Ok(());
        }
        g.retain_fx(e);
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let o = MoveOpts { allow_cancel: false, min: 0, max: Some(2), ..Default::default() };
        let filter = Filter::super_type(SuperType::Energy);
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

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let r = discard(g, me, p, atk, results);
    g.release_fx(atk);
    r
}

fn discard(g: &mut Game, me: CardId, p: usize, atk: EffId, results: &[Res]) -> R {
    let transfers = match results.first() {
        Some(Res::CardsFrom(t)) => *t,
        _ => return Ok(()),
    };
    let n = transfers.len() as i32;
    if let Effect::Attack { damage, .. } = g.e_mut(atk) {
        *damage = 160 + n * 60;
    }
    for (from, c) in transfers.iter() {
        let src = get_target(&g.st, p, *from)?;
        move_cards(g, src.list(), ListRef::Discard(p as u8), &[*c], me)?;
    }
    Ok(())
}
