//! Tool Scrapper (DRX): choose up to 2 Pokémon Tools attached to Pokémon in
//! play (yours or your opponent's) and discard them. Phase 4b (rulings 1778/1853): the prompt can't
//! be cancelled (a cancel was choosing 0).
//!
//! Fixed (phase 4b, F1): the choice is over the Tools (one DiscardEnergyPrompt with a Pokémon Tool
//! filter, min 1, max min(2, Tools in play)), not over Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ToolScrapper", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut tools_in_play = 0usize;
    for (q, pt) in [(p, PlayerType::BottomPlayer), (1 - p, PlayerType::TopPlayer)] {
        for (s, _, _) in for_each_pokemon(g, q, pt).iter() {
            tools_in_play += g.st.slot(q, *s).tools.len();
        }
    }
    if tools_in_play == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Tool as u8), ..Filter::none() };
    let o = MoveOpts { allow_cancel: false, min: 1, max: Some(tools_in_play.min(2) as u8), ..Default::default() };
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_CARD_TO_DISCARD",
        PromptKind::DiscardEnergy { player_type: PlayerType::Any, slots, filter, o },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers = match results.first().copied() {
        Some(Res::CardsFrom(t)) => t,
        _ => return Ok(()),
    };
    for (from, card) in transfers.iter().copied() {
        let t = get_target(&g.st, p, from)?;
        move_cards(g, t.list(), ListRef::Discard(t.p), &[card], me)?;
    }
    Ok(())
}
