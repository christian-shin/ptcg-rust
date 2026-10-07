//! Morpeko (TWM): Snack Seek — once during your turn, look at the top card of
//! your deck; you may discard it. Pick and Stick — attach up to 2 Basic
//! Energy cards from your discard pile to your Pokémon in any way you like.
//!
//! Twinleaf: Snack Seek throws CANNOT_USE_POWER on an empty deck and
//! POWER_ALREADY_USED with the marker, moves the top card to a scratch list,
//! sets the marker and ABILITY_USED, then a ConfirmCardsPrompt (an info
//! prompt, always answered `true` by the oracle → the card is discarded;
//! `null` would put it back on top). Pick and Stick does nothing without a
//! basic Energy in the discard, otherwise a non-cancellable AttachEnergyPrompt
//! (discard → Bench/Active, basic Energy, min 0 max 2: phase 4b R7E, "up to 2"
//! in an attack may take 0, rulings 1721/1778).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Morpeko@TWM",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::POWER, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn snack() -> crate::markers::MarkerName {
    crate::marker!("SNACK_SEARCH_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(snack(), me) {
            m.remove_from(snack(), me);
        }
    }

    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(snack(), me);
            return Ok(());
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(snack(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let top = g.alloc_temp(&[]);
        move_count_from(g, ListRef::Deck(p as u8), top, 1, me)?;
        g.st.players[p].marker.add(snack(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        ability_used(g, p, me);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.l[0] = match top {
            ListRef::Temp(i) => i,
            _ => unreachable!(),
        };
        let id = g.player_id(p);
        g.prompt(id, "DISCARD_FROM_TOP_OF_DECK", PromptKind::ConfirmCards, Cont::Card { card: me, frame: f });
        return Ok(());
    }

    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let has = g.st.players[p].discard.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        });
        if !has {
            return Ok(());
        }
        let mut o = AttachOpts::new(g.st.players[p].discard.len() as u8);
        o.allow_cancel = false;
        o.min = 0;
        o.max = 2;
        let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        slots.push(SlotType::Active as u8);
        let mut f = CardFrame::at(2);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Discard(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let top = ListRef::Temp(f.l[0]);
            let first = results.first().copied().unwrap_or(Res::Null);
            if !first.is_null() {
                let cards: Vec<CardId> = g.lst(top).to_vec();
                move_cards(g, top, ListRef::Discard(p as u8), &cards, me)?;
            } else {
                g.move_to_top_of_destination(top, ListRef::Deck(p as u8));
            }
            Ok(())
        }
        2 => {
            let transfers: SVec<(CardTarget, CardId), 64> = match results.first() {
                Some(Res::Attach(t)) => *t,
                _ => SVec::new(),
            };
            for (to, c) in transfers.iter().copied() {
                let target = get_target(&g.st, p, to)?;
                move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], me)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
