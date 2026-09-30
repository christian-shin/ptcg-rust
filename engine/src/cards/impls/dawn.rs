//! Dawn (M2 / PFL): search your deck for a Basic Pokémon, a Stage 1 Pokémon
//! and a Stage 2 Pokémon, reveal them, put them into your hand, then shuffle.
//!
//! Twinleaf: non-matching deck cards are blocked, `max` is the number of
//! stages present, with `maxBasics` / `maxStage1` / `maxStage2`; ShowCards
//! only when something was taken; no trailing wait after the shuffle.
use super::hilda::reveal_then_shuffle_resume;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dawn", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let (mut basics, mut stage1, mut stage2) = (0u8, 0u8, 0u8);
    let mut opts = ChooseCardsOpts::new(0, 0, false);
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.is_pokemon() && d.stage == Stage::Basic as u8 {
            basics += 1;
        } else if d.is_pokemon() && d.stage == Stage::Stage1 as u8 {
            stage1 += 1;
        } else if d.is_pokemon() && d.stage == Stage::Stage2 as u8 {
            stage2 += 1;
        } else {
            opts.blocked.push(i as u8);
        }
    }
    let (mb, m1, m2) = (basics.min(1), stage1.min(1), stage2.min(1));
    opts.max = mb + m1 + m2;
    opts.max_basics = Some(mb);
    opts.max_stage1 = Some(m1);
    opts.max_stage2 = Some(m2);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    reveal_then_shuffle_resume(g, me, f, results)
}
