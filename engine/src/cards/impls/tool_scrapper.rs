//! Tool Scrapper (DRX): choose up to 2 Pokémon Tools attached to Pokémon in
//! play (yours or your opponent's) and discard them. Phase 4b (rulings 1778/1853): the prompt can't
//! be cancelled (a cancel was choosing 0).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ToolScrapper", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut with_tool = 0u8;
    let mut blocked = TargetList::new();
    for (q, pt) in [(p, PlayerType::BottomPlayer), (1 - p, PlayerType::TopPlayer)] {
        for (s, _, t) in for_each_pokemon(g, q, pt).iter() {
            if !g.st.slot(q, *s).tools.is_empty() {
                with_tool += 1;
            } else {
                blocked.push(*t);
            }
        }
    }
    if with_tool == 0 {
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
        "CHOOSE_POKEMON_TO_DISCARD_CARDS",
        PromptKind::ChoosePokemon { player_type: PlayerType::Any, slots, min: 1, max: with_tool.min(2), allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let sel = results.first().copied().unwrap_or(Res::Null).slots().to_vec();
    for t in sel {
        if let Some(tool) = g.st.slot(t.p as usize, t.s).tools.get(0) {
            move_cards(g, t.list(), ListRef::Discard(t.p), &[tool], me)?;
        }
    }
    Ok(())
}
