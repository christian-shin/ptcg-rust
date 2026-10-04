//! Miraidon (TEF): Peak Acceleration — 40; search your deck for up to 2 Basic
//! Energy and attach them to your Future Pokémon in any way, then shuffle.
//! Sparking Strike — 160.
//!
//! Twinleaf: AttachEnergyPrompt over the deck (Bench + Active, no cancel,
//! min 0, max 2). blockedTo lists every non-Future Pokémon (phase 4b fix: it
//! used to offer them and the Energy then threw INVALID_TARGET). No transfers
//! -> SHUFFLE_DECK. Otherwise each transfer is checked in order
//! (`target.cards[0]` must be a Future Pokémon, else throws INVALID_TARGET
//! after the earlier ones already moved) and the Energy is moved with
//! MOVE_CARDS (no AttachEnergyEffect); then a bare ShuffleDeckPrompt (no
//! trailing wait prompt).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Miraidon", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        slots.push(SlotType::Active as u8);
        let mut o = AttachOpts::new(g.st.players[p].deck.len().min(255) as u8);
        o.allow_cancel = false;
        o.min = 0;
        o.max = 2;
        // Only Future Pokémon can receive the Energy (blockedTo lists the others).
        for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if !g.st.cdef(c).has_tag(tag::FUTURE) {
                o.blocked_to.push(t);
            }
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy {
                cards: ListRef::Deck(p as u8),
                player_type: PlayerType::BottomPlayer,
                slots,
                filter: Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() },
                o,
            },
            Cont::Card { card: me, frame: f },
        );
    }
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
        let first = g.st.slot(target.p as usize, target.s).cards.get(0);
        let future = match first {
            Some(c0) => g.st.cdef(c0).has_tag(tag::FUTURE),
            None => bail!("INVALID_TARGET"),
        };
        if !future {
            bail!("INVALID_TARGET");
        }
        move_cards(g, ListRef::Deck(p as u8), ListRef::Slot(target.p, target.s), &[c], me)?;
    }
    let id = g.player_id(p);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
    Ok(())
}
