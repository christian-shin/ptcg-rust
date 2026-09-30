//! Iron Leaves ex (TEF): Rapid Vernier - when you play this Pokémon from
//! your hand onto your Bench, you may switch it with your Active Pokémon;
//! if you do, you may move any number of Energy from your Benched Pokémon
//! to it. Prism Edge - 180; this Pokémon can't attack during your next turn.
//!
//! Twinleaf quirks kept: the confirm prompt is created before the card is
//! benched (and before the ability-lock probe, which runs in the callback);
//! the energy move loops over every transfer once per transfer, always
//! using the outer transfer's source (moves of cards not in that source are
//! no-ops, but each still reduces a MoveCardsEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "IronLeavesex",
    mask: mask(&[k::PLAY_POKEMON, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            let id = g.player_id(p as usize);
            g.prompt(id, "WANT_TO_USE_ABILITY", PromptKind::Confirm, Cont::Card { card: me, frame: f });
        }
    }
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            if !first.as_bool() {
                return Ok(());
            }
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            // switchPokemon(bench[indexOf(cardList)]): a no-op unless on the Bench.
            if let Some((q, s)) = g.st.find_pokemon_slot(me) {
                if q == p && g.st.players[p].bench_index_of(s).is_some() {
                    crate::engine::turn::switch_pokemon(g, p, s)?;
                }
            }
            let mut blocked_from = SVec::new();
            let mut blocked_to = SVec::new();
            let mut has_energy_on_bench = false;
            let active = g.st.players[p].active;
            for s in g.st.players[p].in_play().iter() {
                let t = if *s == active {
                    CardTarget::new(PlayerType::BottomPlayer, SlotType::Active, 0)
                } else {
                    CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, g.st.players[p].bench_index_of(*s).unwrap() as u8)
                };
                if *s == active {
                    blocked_from.push(t);
                    continue;
                }
                blocked_to.push(t);
                if g.st.slot(p, *s).cards.iter().any(|c| g.st.cdef(c).is_energy()) {
                    has_energy_on_bench = true;
                }
            }
            if !has_energy_on_bench {
                return Ok(());
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            slots.push(SlotType::Active as u8);
            let o = MoveOpts { allow_cancel: false, blocked_from, blocked_to, ..Default::default() };
            let id = g.player_id(p);
            let nf = CardFrame { stage: 2, ..f };
            g.prompt(
                id,
                "MOVE_ENERGY_CARDS",
                PromptKind::MoveEnergy { player_type: PlayerType::BottomPlayer, slots, filter: Filter::super_type(SuperType::Energy), o },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let transfers = match first {
                Res::Transfers(t) => t,
                _ => return Ok(()),
            };
            for (from, _, _) in transfers.iter() {
                let target = ListRef::Slot(p as u8, g.st.players[p].active);
                let source = get_target(&g.st, p, *from)?;
                for (_, _, card) in transfers.iter() {
                    move_cards(g, source.list(), target, &[*card], me)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
