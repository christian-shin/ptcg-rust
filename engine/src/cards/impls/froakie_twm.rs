//! Froakie (TWM): Flock — search your deck for up to 2 Froakie and put them
//! onto your Bench, then shuffle. Flop — 10.
//!
//! Twinleaf: same shape as Chatot's A Capella (no empty-deck / full-bench
//! check; the prompt max is min(empty bench slots, 2)), with a name filter.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Froakie@Froakie TWM", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let open = empty_bench_slots(g, p);
        let max = open.len().min(2) as u8;
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        for (i, s) in open.iter().enumerate().take(3) {
            f.a[1 + i] = *s as i32;
        }
        f.l[0] = open.len() as u8;
        let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), name: Some("Froakie"), ..Filter::none() };
        choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, max, false), Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    super::chatot::bench_search_resume(g, f, results)
}
