//! Smoochum (SSP): Delightful Kiss ("Happy Kiss") — search your deck for up to
//! 2 Basic [P] Energy cards and attach them to 1 of your Benched Pokémon.
//! Then, shuffle your deck.
//!
//! Fixed (phase 4b, W4): the prompt let the two Energy go to different
//! Benched Pokémon; it now requires the same target (`sameTarget`).
//!
//! Twinleaf: an empty deck makes the attack do nothing (it is still usable:
//! phase 4b R7E, rulings 337 and 1790; it used to throw CANNOT_USE_ATTACK).
//! Opens an AttachEnergyPrompt (0..2, no cancel) and, without waiting for it, a
//! ShuffleDeckPrompt whose callback applies the order (no trailing wait). The
//! attach callback shuffles again (SHUFFLE_DECK) when nothing was attached.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Smoochum", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let mut o = AttachOpts::new(g.st.players[p].deck.len() as u8);
        o.allow_cancel = false;
        o.min = 0;
        o.max = 2;
        o.same_target = true;
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Psychic Energy"), ..Filter::none() };
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Deck(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
        g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers: SVec<(CardTarget, CardId), 64> = match results.first().copied().unwrap_or(Res::Null) {
        Res::Attach(t) => t,
        _ => SVec::new(),
    };
    if transfers.is_empty() {
        shuffle_deck(g, p);
        return Ok(());
    }
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Deck(p as u8), target.list(), &[c], me)?;
    }
    Ok(())
}
