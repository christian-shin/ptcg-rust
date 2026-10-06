//! Mega Gengar ex (MBG / PFL): Shadowy Concealment — if 1 of your [D]
//! Pokémon is Knocked Out by damage from an attack from your opponent's
//! Pokémon ex, that player takes 1 fewer Prize card (doesn't stack).
//! Void Gale — 230; move an Energy from this Pokémon to 1 of your Benched
//! Pokémon.
//!
//! Twinleaf: the KnockOutEffect branch runs before the core prize
//! adjustments (so it lowers the base count 1 → 0), only during the
//! opponent's ATTACK phase, with this card in play for the KO'd player and
//! not ability-blocked (probe for the *opponent*), a [D] target (by
//! CheckPokemonTypeEffect) and an ex attacker (fixed in phase 4b, F1: the Pokémon that used the attack, not the
//! opponent's Active at the Knock Out check, which can be another Pokémon after a switch); a slot marker keeps it
//! from stacking (fixed in phase 4b: the check looked for the marker of this
//! copy only, so a second copy never saw the first one's marker; it now
//! tests the marker by name). Void Gale sets a player marker; on the AfterAttackEffect
//! with that marker and any benched Pokémon a non-cancellable
//! AttachEnergyPrompt moves 1 Energy from the Active to the Bench.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "MegaGengarex",
    mask: mask(&[k::KNOCK_OUT, k::ATTACK, k::AFTER_ATTACK, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn void_gale() -> crate::markers::MarkerName {
    crate::marker!("VOID_GALE_MARKER")
}

fn non_stack() -> crate::markers::MarkerName {
    crate::marker!("MEGA_GENGAR_SHADOW_HIDING_APPLIED")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::KnockOut { p, target, .. } = *g.e(e) {
        let player = p as usize;
        let opponent = 1 - player;
        if g.st.phase != GamePhase::Attack || g.st.active_player as usize != opponent {
            return Ok(());
        }
        let in_play = for_each_pokemon(g, player, PlayerType::BottomPlayer).iter().any(|x| x.1 == me);
        if !in_play || is_ability_blocked(g, opponent, me, None) {
            return Ok(());
        }
        let types = crate::engine::game_effect::pokemon_types(g, target);
        let (te, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
        let dark = match te {
            Effect::CheckPokemonType { card_types, .. } => card_types.contains(&ct::DARK),
            _ => false,
        };
        if !dark {
            return Ok(());
        }
        // The Pokémon that used the attack, wherever it is by now (switched to the Bench, ...).
        let ex = g.attacker_of_knock_out(player).and_then(|a| a.0).map(|c| g.st.cdef(c).has_tag(tag::POKEMON_EX_LOWER)).unwrap_or(false);
        if !ex {
            return Ok(());
        }
        let m = &mut g.st.players[target.p as usize].slots[target.s as usize].marker;
        if m.has(non_stack()) {
            return Ok(());
        }
        m.add(non_stack(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        if let Effect::KnockOut { prize_count, .. } = g.e_mut(e) {
            if *prize_count > 0 {
                *prize_count -= 1;
            }
        }
    }
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.st.players[p].marker.add(void_gale(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        return Ok(());
    }
    if let Effect::AfterAttack { p, .. } = *g.e(e) {
        let p = p as usize;
        if g.st.players[p].marker.has_from(void_gale(), me) {
            let pl = &g.st.players[p];
            if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
                return Ok(());
            }
            let a = pl.active;
            let mut o = AttachOpts::new(pl.slots[a as usize].cards.len() as u8);
            o.allow_cancel = false;
            o.min = 1;
            o.max = 1;
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            f.a[1] = a as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "ATTACH_ENERGY_TO_BENCH",
                PromptKind::AttachEnergy { cards: ListRef::Slot(p as u8, a), player_type: PlayerType::BottomPlayer, slots, filter: Filter::super_type(SuperType::Energy), o },
                Cont::Card { card: me, frame: f },
            );
            return Ok(());
        }
    }
    remove_marker_at_end_of_turn(g, e, void_gale(), me);
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let a = f.a[1] as SlotId;
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Slot(p as u8, a), target.list(), &[c], me)?;
    }
    Ok(())
}
