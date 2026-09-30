//! Bianca's Devotion (TEF): heal all damage from 1 of your Pokémon that has
//! 30 HP or less remaining.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BiancasDevotion", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let mut blocked = TargetList::new();
    let mut valid = false;
    for (s, _, target) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
        let hp = crate::engine::check::check_hp(g, p, *s)?;
        if hp - g.st.slot(p, *s).damage > 30 {
            blocked.push(*target);
        } else {
            valid = true;
        }
    }
    if !valid {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
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
    let t = match results.first().and_then(|r| r.slots().first().copied()) {
        Some(t) => t,
        None => return Ok(()),
    };
    let damage = g.st.slot(t.p as usize, t.s).damage;
    g.run_fx(Effect::Heal { p: p as u8, target: t, damage })?;
    Ok(())
}
