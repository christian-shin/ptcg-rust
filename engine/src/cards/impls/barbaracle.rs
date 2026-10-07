//! Barbaracle (M3 / POR 43): Stone Arms — once during your turn, attach a
//! Basic [F] Energy from your hand to 1 of your [F] Pokémon. Hammer In — 80.
//!
//! Twinleaf: the Ability throws CANNOT_USE_POWER without a Basic 'Fighting
//! Energy' in the hand (phase 4b R7E: it only checked for a non-empty hand) and
//! the AttachEnergyPrompt takes exactly 1 card (it allowed 0, with the ability
//! used up whatever the answer; ruling 1853); the ability counts as used
//! (marker + board effect) once the prompt is answered.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "Barbaracle",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn stone_arms() -> crate::markers::MarkerName {
    marker!("STONE_ARMS_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(stone_arms(), me);
        }
    }
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(stone_arms(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let has_fighting = g.st.players[p].hand.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.name == "Fighting Energy"
        });
        if !has_fighting {
            bail!("CANNOT_USE_POWER");
        }
        let mut o = AttachOpts::new(g.st.players[p].hand.len() as u8);
        o.allow_cancel = false;
        o.min = 1;
        o.max = 1;
        for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if !g.st.cdef(c).card_type.contains(&ct::FIGHTING) {
                o.blocked_to.push(t);
            }
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        slots.push(SlotType::Active as u8);
        let mut filter = Filter::super_type(SuperType::Energy);
        filter.energy_type = Some(EnergyType::Basic as u8);
        filter.name = Some("Fighting Energy");
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Hand(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(stone_arms(), me) {
            m.remove_from(stone_arms(), me);
        }
    }
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
    ability_used(g, p, me);
    g.st.players[p].marker.add(stone_arms(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Hand(p as u8), target.list(), &[c], me)?;
    }
    Ok(())
}
