//! Marnie's Grimmsnarl ex (DRI): Punk Up — when you evolve into this Pokémon
//! from your hand, you may search your deck for up to 5 Basic [D] Energy and
//! attach them to your Marnie's Pokémon, then shuffle. Shadow Bullet — 180,
//! and 30 damage to 1 of the opponent's Benched Pokémon.
//!
//! Twinleaf: fires on any EvolveEffect for this card (Rare Candy included);
//! the Marnie's check (blockedTo) is taken before the evolution, so the
//! evolving slot is judged by the Pokémon it evolves from; an empty deck skips
//! the ability; a cancelled or empty attach uses SHUFFLE_DECK (with wait),
//! otherwise the shuffle has no wait.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MarniesGrimmsnarlex", mask: mask(&[k::EVOLVE, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::Evolve { p, card, .. } = *g.e(e) {
        if card != me {
            return Ok(());
        }
        let p = p as usize;
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        // blockedTo, kept in the frame as a bitmask (bit 0 = Active, 1 + i = Bench i).
        let mut mask_bits = 0i32;
        for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if !g.st.cdef(c).has_tag(tag::MARNIES) {
                let bit = if t.slot == SlotType::Active { 0 } else { 1 + t.index as i32 };
                mask_bits |= 1 << bit;
            }
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = mask_bits;
        confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let o = 1 - p;
        let pl = &g.st.players[o];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(3);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if !first.as_bool() {
                return Ok(());
            }
            let p = f.a[0] as usize;
            let mut o = AttachOpts::new(g.st.players[p].deck.len() as u8);
            o.allow_cancel = true;
            o.min = 0;
            o.max = 5;
            for bit in 0..9 {
                if f.a[1] & (1 << bit) != 0 {
                    let t = if bit == 0 {
                        CardTarget::new(PlayerType::BottomPlayer, SlotType::Active, 0)
                    } else {
                        CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, (bit - 1) as u8)
                    };
                    o.blocked_to.push(t);
                }
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            slots.push(SlotType::Active as u8);
            let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Darkness Energy"), ..Filter::none() };
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "ATTACH_ENERGY_CARDS",
                PromptKind::AttachEnergy { cards: ListRef::Deck(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let p = f.a[0] as usize;
            let transfers: SVec<(CardTarget, CardId), 64> = match first {
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
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
            Ok(())
        }
        3 => {
            let atk = f.e[0];
            let sel = first.slots();
            if sel.is_empty() {
                g.release_fx(atk);
                return Ok(());
            }
            let t = sel[0];
            let r = put_damage(g, atk, 30, t);
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}
