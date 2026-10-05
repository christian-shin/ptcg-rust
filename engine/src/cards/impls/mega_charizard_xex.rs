//! Mega Charizard X ex (M2 / PFL): Inferno X — discard any amount of [R]
//! Energy from among your Pokémon; 90 damage for each card discarded.
//!
//! Twinleaf: counts every attached Energy card whose static `provides`
//! contains FIRE or ANY, blocks (by index in the slot's card list) the other
//! Energy cards, and asks a non-cancellable DiscardEnergyPrompt for 1..=count.
//! Each chosen card is moved to the discard pile with MOVE_CARDS, and the
//! damage is set to 90 x the number chosen (left at 90 if the prompt returned
//! null). Without any such Energy (a copycat) there is no prompt and the damage
//! is 0 (phase 4b).
//! R7A (ruling 1874): the Energy is chosen first, the damage is done, then the Energy is discarded (`move_cards_after_damage`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaCharizardXex@Mega Charizard X ex M2", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn gives_fire(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    d.provides.contains(&ct::FIRE) || d.provides.contains(&ct::ANY)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let mut total = 0usize;
    let mut o = MoveOpts { allow_cancel: false, ..Default::default() };
    for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let mut b = Blocked::default();
        let mut any = false;
        for (i, c) in g.st.slot(p, s).cards.iter().enumerate() {
            if g.st.cdef(c).is_energy() {
                if gives_fire(g, c) {
                    total += 1;
                } else {
                    b.push(i as u8);
                    any = true;
                }
            }
        }
        if any {
            o.blocked_map.push((t, b));
        }
    }
    // Phase 4b (Y2-4): nothing to discard (a copied Inferno X, the copycat has no [R]
    // Energy): no damage, and no prompt without a valid answer.
    if total == 0 {
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 0;
        }
        return Ok(());
    }
    o.min = 1;
    o.max = Some(total as u8);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.e[0] = e;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_ENERGIES_TO_DISCARD",
        PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter: Filter::super_type(SuperType::Energy), o },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let first = results.first().copied().unwrap_or(Res::Null);
    let r = (|| -> R {
        let transfers = match first {
            Res::CardsFrom(t) => t,
            _ => return Ok(()),
        };
        for (from, c) in transfers.iter().copied() {
            let s = get_target(&g.st, p, from)?;
            move_cards_after_damage(g, atk, s.list(), ListRef::Discard(p as u8), &[c], me)?;
            if let Effect::Attack { damage, .. } = g.e_mut(atk) {
                *damage = transfers.len() as i32 * 90;
            }
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
