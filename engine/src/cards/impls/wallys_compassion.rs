//! Wally's Compassion (MEG / M1S): heal all damage from 1 of your Mega
//! Evolution Pokémon ex; if you do, put all Energy attached to it into your
//! hand.
//!
//! Twinleaf: only damaged Mega ex are selectable (the rest are blocked);
//! HealEffect for the slot's full damage, then every Energy card in the slot
//! (checked after the heal) moves to the hand in one MOVE_CARDS, whether or
//! not the heal did anything.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "WallysCompassion", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let mut blocked = SVec::new();
    let mut has = false;
    for (s, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let d = g.st.cdef(c);
        if d.has_tag(tag::POKEMON_SV_MEGA) && d.has_tag(tag::POKEMON_EX_LOWER) && g.st.slot(p, s).damage > 0 {
            has = true;
        } else {
            blocked.push(t);
        }
    }
    if !has {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
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

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let t = match results.first().and_then(|r| r.slots().first().copied()) {
        Some(t) => t,
        None => return Ok(()),
    };
    let damage = g.st.slot(t.p as usize, t.s).damage;
    g.run_fx(Effect::Heal { p: p as u8, target: t, damage })?;
    let energy: Vec<CardId> = g.st.slot(t.p as usize, t.s).cards.iter().filter(|c| g.st.cdef(*c).is_energy()).collect();
    move_cards(g, t.list(), ListRef::Hand(p as u8), &energy, me)?;
    Ok(())
}
