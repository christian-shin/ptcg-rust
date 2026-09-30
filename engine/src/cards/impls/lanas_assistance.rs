//! Lana's Aid (TWM, supporter): put up to 3 in any combination of Pokémon
//! without a Rule Box and Basic Energy cards from your discard pile into
//! your hand.
//!
//! Twinleaf: DiscardToHandEffect is probed before the supporter check; the
//! card moves to the supporter pile before the "no target" failure; the
//! cards are shown to the opponent and then moved.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LanasAssistance", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let (_, prevented) = g.run_fx(Effect::DiscardToHand { p: p as u8, card: me })?;
    if prevented {
        return Ok(());
    }
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let mut n = 0;
    let mut opts = ChooseCardsOpts::new(1, 3, false);
    let discard: Vec<CardId> = g.st.players[p].discard.iter().collect();
    for (i, c) in discard.iter().enumerate() {
        let d = g.st.cdef(*c);
        let non_rule_pokemon = d.is_pokemon() && !d.has_rule_box();
        let basic_energy = d.is_energy() && d.energy_type == EnergyType::Basic as u8;
        if non_rule_pokemon || basic_energy {
            n += 1;
        } else {
            opts.blocked.push(i as u8);
        }
    }
    if n == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Discard(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    let cards: Vec<CardId> = first.cards().to_vec();
    show_cards_to_player(g, 1 - p, cards.len());
    move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), &cards, me)?;
    Ok(())
}
