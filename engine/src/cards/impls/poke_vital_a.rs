//! Poké Vital A (SFA, ACE SPEC): heal 150 damage from 1 of your Pokémon.
//!
//! Twinleaf: undamaged Pokémon are blocked; no cancel; HealEffect 150. Phase
//! 4b: "This card can't be put into your hand or deck from the discard pile"
//! is the same MoveCardsEffect clause as Neutralization Zone (filters this
//! card out of a move from its owner's discard pile to hand or deck, and
//! prevents the move when nothing is left).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PokeVitalA", mask: mask(&[k::TRAINER, k::MOVE_CARDS]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::MoveCards { source, destination, cards, count, .. } = *g.e(e) {
        for p in 0..2usize {
            if source != ListRef::Discard(p as u8) || !g.st.players[p].discard.iter().any(|c| c == me) {
                continue;
            }
            if destination != ListRef::Hand(p as u8) && destination != ListRef::Deck(p as u8) {
                continue;
            }
            let v: Vec<CardId>;
            let new_count;
            if let Some(cs) = cards {
                if !cs.iter().any(|c| c == me) {
                    continue;
                }
                v = cs.iter().filter(|c| *c != me).collect();
                new_count = count;
            } else if let Some(n) = count {
                v = g.st.players[p].discard.iter().filter(|c| *c != me).take(n.max(0) as usize).collect();
                new_count = None;
            } else {
                v = g.st.players[p].discard.iter().filter(|c| *c != me).collect();
                new_count = None;
            }
            let prevent = v.is_empty();
            if let Effect::MoveCards { cards, count, .. } = g.e_mut(e) {
                *cards = Some(List::from_slice(&v));
                *count = new_count;
            }
            if prevent {
                g.set_prevent(e, true);
            }
        }
        return Ok(());
    }
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut has = false;
    let mut blocked: TargetList = SVec::new();
    for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.slot(p, s).damage == 0 {
            blocked.push(t);
        } else {
            has = true;
        }
    }
    if !has {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_HEAL",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    for t in targets {
        g.run_fx(Effect::Heal { p: p as u8, target: t, damage: 150 })?;
    }
    Ok(())
}
