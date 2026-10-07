//! Whimsicott ex (SV11W 5): Energy Gift — search your deck for up to 3
//! Basic Energy cards and attach them to your Pokémon in any way you like,
//! then shuffle. Wonder Cotton — your opponent reveals their hand; 50
//! damage for each Trainer card there.
//!
//! Twinleaf quirks kept: an empty deck skips Energy Gift; the generator is
//! resumed inside the transfer loop, so with no transfer the deck is never
//! shuffled, and otherwise the ShuffleDeckPrompt (no wait) is created after
//! the first MOVE_CARDS (the rest follow before it resolves). Wonder Cotton
//! always shows the opponent's hand (even when empty) and sets
//! `effect.damage` when the prompt resolves.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Whimsicottex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, source) = match *g.e(e) {
            Effect::Attack { p, source, .. } => (p as usize, source),
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        let mut o = AttachOpts::new(g.st.players[p].deck.len() as u8);
        o.allow_cancel = false;
        o.min = 0;
        o.max = 3;
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        slots.push(SlotType::Active as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.l[0] = source.s;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Deck(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(2);
        f.e[0] = e;
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let transfers: SVec<(CardTarget, CardId), 64> = match results.first() {
                Some(Res::Attach(t)) => *t,
                _ => SVec::new(),
            };
            for (i, (to, c)) in transfers.iter().copied().enumerate() {
                let target = get_target(&g.st, p, to)?;
                let src = g.st.slot_pokemon(p, f.l[0]).unwrap_or(NO_CARD);
                move_cards(g, ListRef::Deck(p as u8), target.list(), &[c], src)?;
                if i == 0 {
                    let id = g.player_id(p);
                    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
                }
            }
            Ok(())
        }
        2 => {
            let atk = f.e[0];
            let o = 1 - p;
            let n = g.st.players[o].hand.iter().filter(|c| g.st.cdef(*c).is_trainer()).count() as i32;
            if let Effect::Attack { damage, .. } = g.e_mut(atk) {
                *damage = n * 50;
            }
            g.release_fx(atk);
            Ok(())
        }
        _ => Ok(()),
    }
}
