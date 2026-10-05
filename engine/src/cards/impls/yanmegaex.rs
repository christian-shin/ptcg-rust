//! Yanmega ex (DRI): Buzz Boost — once during your turn, when this Pokémon
//! moves from your Bench to the Active Spot, you may search your deck for up
//! to 3 Basic [G] Energy and attach them to this Pokémon, then shuffle.
//! Jet Cyclone — 210; move 3 Energy from this Pokémon to 1 of your Benched
//! Pokémon.
//!
//! Twinleaf: on MovedToActiveEffect for this card during its owner's turn
//! (and listed in movedToActiveThisTurn), unless the player marker is set, a
//! ConfirmPrompt. No → marker. Yes → a 'test' PowerEffect probe (blocked →
//! nothing, no marker), marker, then a non-cancellable AttachEnergyPrompt
//! (deck → Active, basic 'Grass Energy', min 0 max 3). No transfer →
//! SHUFFLE_DECK; otherwise MOVE_CARDS + SHUFFLE_DECK per transfer (quirk
//! kept: one shuffle per card). Jet Cyclone: AttachEnergyPrompt (Active →
//! Bench, any Energy, sameTarget, no cancel), MOVE_CARDS each. Phase 4b fix:
//! with no Benched Pokémon the attack does nothing more (the prompt, min 3,
//! was unanswerable), and min = max = min(3, Energy attached) so a Pokémon
//! with fewer than 3 Energy cards can't get stuck either.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Yanmegaex",
    mask: mask(&[k::END_TURN, k::MOVED_TO_ACTIVE, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn buzz() -> crate::markers::MarkerName {
    crate::marker!("BUZZ_BOOST_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    remove_marker_at_end_of_turn(g, e, buzz(), me);

    if let Effect::MovedToActive { p, card } = *g.e(e) {
        let p = p as usize;
        if card == me && g.st.active_player as usize == p && g.st.players[p].moved_to_active_this_turn.contains(&me) {
            if g.st.players[p].marker.has_from(buzz(), me) {
                return Ok(());
            }
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
        }
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let has_bench = g.st.players[p].bench.iter().any(|s| !g.st.players[p].slots[*s as usize].cards.is_empty());
        if !has_bench {
            return Ok(());
        }
        let a = g.st.players[p].active;
        let energy_count = g.st.slot(p, a).cards.iter().filter(|c| g.st.cdef(*c).is_energy()).count();
        let move_count = energy_count.min(3) as u8;
        let mut o = AttachOpts::new(g.st.slot(p, a).cards.len() as u8);
        o.allow_cancel = false;
        o.min = move_count;
        o.max = move_count;
        o.same_target = true;
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(3);
        f.a[0] = p as i32;
        f.e[0] = e;
        g.retain_fx(e);
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Slot(p as u8, a), player_type: PlayerType::BottomPlayer, slots, filter: Filter::super_type(SuperType::Energy), o },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if !first.as_bool() {
                g.st.players[p].marker.add(buzz(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
                return Ok(());
            }
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            g.st.players[p].marker.add(buzz(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            let mut o = AttachOpts::new(g.st.players[p].deck.len() as u8);
            o.allow_cancel = false;
            o.min = 0;
            o.max = 3;
            let filter = Filter {
                super_type: Some(SuperType::Energy as u8),
                energy_type: Some(EnergyType::Basic as u8),
                name: Some("Grass Energy"),
                ..Filter::none()
            };
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "ATTACH_ENERGY_TO_BENCH",
                PromptKind::AttachEnergy { cards: ListRef::Deck(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let transfers: SVec<(CardTarget, CardId), 16> = match first {
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
                shuffle_deck(g, p);
            }
            Ok(())
        }
        3 => {
            let transfers: SVec<(CardTarget, CardId), 16> = match first {
                Res::Attach(t) => t,
                _ => SVec::new(),
            };
            let src = ListRef::Slot(p as u8, g.st.players[p].active);
            let atk = f.e[0];
            let r = (|| -> R {
                for (to, c) in transfers.iter().copied() {
                    let target = get_target(&g.st, p, to)?;
                    move_cards_after_damage(g, atk, src, target.list(), &[c], me)?;
                }
                Ok(())
            })();
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}
