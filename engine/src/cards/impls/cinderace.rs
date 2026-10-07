//! Cinderace (M1L): Explosiveness (setup, core `PLAY_DURING_SETUP` tag).
//! Flame Turbo — 50; search your deck for up to 3 Basic Energy cards and
//! attach them to your Benched Pokémon in any way you like, then shuffle.
//!
//! Twinleaf: the AttachEnergyPrompt (deck, Bench, min 0 / max 3, no cancel) is followed
//! immediately by a SHUFFLE_DECK (before the answer); an empty answer shuffles
//! again, otherwise a MOVE_CARDS per transfer.
//!
//! Fixed (phase 4b, W4): with an empty deck the attack threw
//! CANNOT_USE_ATTACK (unusable, no damage); it now just does its 50 damage.
//! Fixed (phase 4b, F1): with no Benched Pokémon the prompt had no target; the attack now does its
//! damage and nothing else (rulings 1790, 336).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Cinderace", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let pl = &g.st.players[p];
    if pl.deck.is_empty() || !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
        return Ok(());
    }
    let mut o = AttachOpts::new(g.st.players[p].deck.len() as u8);
    o.allow_cancel = false;
    o.min = 0;
    o.max = 3;
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_TO_BENCH",
        PromptKind::AttachEnergy { cards: ListRef::Deck(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
        Cont::Card { card: me, frame: f },
    );
    shuffle_deck(g, p);
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers: SVec<(CardTarget, CardId), 64> = match results.first() {
        Some(Res::Attach(t)) => *t,
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
