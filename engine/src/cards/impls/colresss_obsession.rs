//! Colress's Tenacity (SFA): search your deck for a Stadium card and an
//! Energy card, reveal them, put them into your hand, shuffle.
//!
//! Twinleaf: every deck card that is neither a Stadium nor an Energy is
//! blocked; `max = min(stadiums,1) + min(energies,1)` with `maxTrainers` /
//! `maxEnergies`; ShowCards only when something was taken; the final shuffle
//! prompt has no wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ColresssObsession", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    // Fixed (phase 4b, rulings 779/851): a search of an empty deck is not possible, so the card can't be played.
    if g.st.players[p].deck.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let (mut trainers, mut energies) = (0u8, 0u8);
    let mut opts = ChooseCardsOpts::new(0, 0, false);
    for (i, c) in g.st.players[p].deck.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.super_type == SuperType::Trainer as u8 && d.trainer_type == TrainerType::Stadium as u8 {
            trainers += 1;
        } else if d.is_energy() {
            energies += 1;
        } else {
            opts.blocked.push(i as u8);
        }
    }
    let (mt, mn) = (trainers.min(1), energies.min(1));
    opts.max = mt + mn;
    opts.max_trainers = Some(mt);
    opts.max_energies = Some(mn);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    super::hilda::reveal_then_shuffle_resume(g, me, f, results)
}
