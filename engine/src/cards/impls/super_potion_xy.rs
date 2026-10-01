//! Super Potion (XY, as Super Potion JTG): heal 60 damage from 1 of your
//! Pokémon. If you do, discard an Energy attached to that Pokémon.
//!
//! Twinleaf: Pokémon that are undamaged or have no Energy card in `cards`
//! are blocked; both prompts can be cancelled (nothing happens); the Energy
//! choice is on the whole slot list (superType ENERGY); MOVE_CARDS to the
//! discard, then HealEffect 60 (x-and-y file; same flow as the BS port).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SuperPotion@JTG", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut has = false;
    let mut blocked: TargetList = SVec::new();
    for (s, _, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let slot = g.st.slot(p, s);
        if slot.damage == 0 || !slot.cards.iter().any(|c| g.st.cdef(c).is_energy()) {
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
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: true, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let t = match first.slots().first() {
                Some(t) => *t,
                None => return Ok(()),
            };
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.l[0] = t.p;
            nf.l[1] = t.s;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_CARD_TO_DISCARD",
                PromptKind::ChooseCards { cards: ListRef::Slot(t.p, t.s), filter: Filter::super_type(SuperType::Energy), opts: ChooseCardsOpts::new(1, 1, true) },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if cards.is_empty() {
                return Ok(());
            }
            let t = SlotRef { p: f.l[0], s: f.l[1] };
            move_cards(g, ListRef::Slot(t.p, t.s), ListRef::Discard(p as u8), &cards, me)?;
            g.run_fx(Effect::Heal { p: p as u8, target: t, damage: 60 })?;
            Ok(())
        }
        _ => Ok(()),
    }
}
