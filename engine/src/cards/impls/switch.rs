//! Switch (BS): switch your Active Pokémon with 1 of your Benched Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Switch", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        let pl = &g.st.players[p];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        g.set_prevent(e, true);
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let id = g.player_id(p);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_SWITCH",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let sel = results.first().map(|r| *r).unwrap_or(Res::Null);
    let t = match sel.slots().first() {
        Some(t) => *t,
        None => return Ok(()),
    };
    let a = g.st.players[p].active;
    crate::engine::game_effect::clear_effects(&mut g.st.players[p].slots[a as usize]);
    if t.p as usize == p {
        crate::engine::turn::switch_pokemon(g, p, t.s)?;
    }
    Ok(())
}
