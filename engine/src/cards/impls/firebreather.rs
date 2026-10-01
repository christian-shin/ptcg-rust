//! Firebreather (M2 / PFL): search your deck for up to 7 Basic [R] Energy
//! cards, reveal them, put them into your hand, shuffle.
//!
//! Twinleaf: the empty-deck check comes before the Supporter-played check;
//! the filter is Basic Energy named "Fire Energy"; ShowCards only when
//! something was taken; the final shuffle prompt has no wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Firebreather", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Fire Energy"), ..Filter::none() };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 7, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    super::hilda::reveal_then_shuffle_resume(g, me, f, results)
}
