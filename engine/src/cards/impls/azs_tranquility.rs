//! AZ's Tranquility (CRI / M4): switch your Active Pokémon with 1 of your
//! Benched Pokémon; if you moved a Pokémon ex to the Bench, heal 80 from it.
//!
//! Twinleaf: the Supporter moves to the supporter pile with preventDefault,
//! then a mandatory ChoosePokemon prompt on the Bench; the callback switches
//! and heals the previous Active (any card tagged ex in the slot) by 80.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "AzsTranquility", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let has_bench = {
        let pl = &g.st.players[p];
        pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty())
    };
    if !has_bench {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_NEW_ACTIVE_POKEMON",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
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
    let prev = g.st.players[p].active;
    crate::engine::turn::switch_pokemon(g, p, t.s)?;
    let ex = g.st.slot(p, prev).cards.iter().any(|c| g.st.cdef(c).has_tag(tag::POKEMON_EX_LOWER));
    if ex {
        g.run_fx(Effect::Heal { p: p as u8, target: SlotRef::new(p, prev), damage: 80 })?;
    }
    Ok(())
}
