//! Toxtricity (M2 / PFL 68): Sinister Surge — once during your turn, search
//! your deck for a Basic [D] Energy and attach it to 1 of your [D] Pokémon,
//! then shuffle; put 2 damage counters on that Pokémon. Thwap — 100.
//!
//! Twinleaf: ABILITY_USED and the once-per-turn marker are set in the prompt
//! callback (so a cancelled prompt still uses the ability); SHUFFLE_DECK runs
//! before the 20 damage is added directly (`target.damage += 20`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Toxtricity",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn bad_boost() -> crate::markers::MarkerName {
    crate::marker!("BAD_BOOST_MARKER")
}

fn dark_energy_filter() -> Filter {
    Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Darkness Energy"), ..Filter::none() }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(bad_boost(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(bad_boost(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let mut o = AttachOpts::new(g.st.players[p].deck.len() as u8);
        o.allow_cancel = true;
        o.min = 0;
        o.max = 1;
        for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if !g.st.cdef(c).card_type.contains(&ct::DARK) {
                o.blocked_to.push(t);
            }
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        slots.push(SlotType::Active as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Deck(p as u8), player_type: PlayerType::BottomPlayer, slots, filter: dark_energy_filter(), o },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(bad_boost(), me) {
            m.remove_from(bad_boost(), me);
        }
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
    ability_used(g, p, me);
    g.st.players[p].marker.add(bad_boost(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    if transfers.is_empty() {
        shuffle_deck(g, p);
        return Ok(());
    }
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Deck(p as u8), target.list(), &[c], me)?;
        shuffle_deck(g, p);
        g.st.players[target.p as usize].slots[target.s as usize].damage += 20;
    }
    Ok(())
}
