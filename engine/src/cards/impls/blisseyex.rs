//! Blissey ex (TWM): Happy Switch — once during your turn, you may move a
//! Basic Energy from 1 of your Pokémon to another of your Pokémon.
//! Return — 180; you may draw until you have 6 cards in hand.
//!
//! Twinleaf: the once-per-turn marker (BLISSFUL_SWAP_MARKER) and
//! ABILITY_USED are set inside the transfer loop; Return's ConfirmPrompt
//! draws one card at a time with MOVE_CARDS.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Blisseyex",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn swap_marker() -> crate::markers::MarkerName {
    crate::marker!("BLISSFUL_SWAP_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(swap_marker(), me);
            return Ok(());
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(swap_marker(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let mut has_basic = false;
        let mut count = 0;
        for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            count += 1;
            if g.st.slot(p, s).cards.iter().any(|c| {
                let d = g.st.cdef(c);
                d.is_energy() && d.energy_type == EnergyType::Basic as u8
            }) {
                has_basic = true;
            }
        }
        if !has_basic || count <= 1 {
            bail!("CANNOT_USE_POWER");
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let mut filter = Filter::super_type(SuperType::Energy);
        filter.energy_type = Some(EnergyType::Basic as u8);
        let o = MoveOpts { allow_cancel: false, min: 1, max: Some(1), ..Default::default() };
        let id = g.player_id(p);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        g.prompt(id, "MOVE_ENERGY_CARDS", PromptKind::MoveEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o }, Cont::Card { card: me, frame: f });
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(swap_marker(), me) {
            m.remove_from(swap_marker(), me);
        }
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut f = CardFrame::at(2);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(id, "WANT_TO_DRAW_UNTIL_6", PromptKind::Confirm, Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let transfers = match first {
                Res::Transfers(t) => t,
                _ => return Ok(()),
            };
            for (from, to, card) in transfers.iter() {
                g.st.players[p].marker.add(swap_marker(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
                ability_used(g, p, me);
                let src = get_target(&g.st, p, *from)?;
                let dst = get_target(&g.st, p, *to)?;
                move_cards(g, src.list(), dst.list(), &[*card], me)?;
            }
            Ok(())
        }
        2 => {
            if first.as_bool() {
                while g.st.players[p].hand.len() < 6 {
                    if g.st.players[p].deck.is_empty() {
                        break;
                    }
                    move_count_from(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), 1, me)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
