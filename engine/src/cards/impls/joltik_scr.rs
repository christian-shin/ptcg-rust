//! Joltik (SCR): Jolting Charge — search your deck for up to 2 Basic [G]
//! Energy and up to 2 Basic [L] Energy and attach them to your Pokémon in
//! any way you like, then shuffle.
//!
//! Twinleaf: a cancellable AttachEnergyPrompt on the deck (basic Energy,
//! max 4, differentTypes, validCardTypes [G, L], maxPerType 2). No transfer
//! → SHUFFLE_DECK; otherwise a MOVE_CARDS per transfer, then a
//! ShuffleDeckPrompt.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Joltik@SCR", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let mut o = AttachOpts::new(g.st.players[p].deck.len() as u8);
    o.allow_cancel = true;
    o.min = 0;
    o.max = 4;
    o.different_types = true;
    let mut vt = SVec::new();
    vt.push(ct::GRASS);
    vt.push(ct::LIGHTNING);
    o.valid_card_types = Some(vt);
    o.max_per_type = Some(2);
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    slots.push(SlotType::Active as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_CARDS",
        PromptKind::AttachEnergy { cards: ListRef::Deck(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
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
    shuffle_deck(g, p);
    Ok(())
}
