//! Eelektrik (SV11B): Dynamotor — once during your turn, you may attach a
//! [L] Energy card from your discard pile to 1 of your Benched Pokémon.
//! Electric Ball — 50.
//!
//! Twinleaf: throws CANNOT_USE_POWER without a Benched Pokémon or a basic
//! Energy providing [L] in the discard, POWER_ALREADY_USED with the marker;
//! then a cancellable AttachEnergyPrompt (discard → Bench, basic 'Lightning
//! Energy', min 1 max 1). The marker is set only when a transfer is made
//! (no ABILITY_USED board effect). PlayPokemonEffect of this card and every
//! EndTurnEffect clear the marker.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Eelektrik@BLK",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::POWER]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn dynamotor() -> crate::markers::MarkerName {
    crate::marker!("DYNAMOTOR_MAREKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(dynamotor(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            bail!("CANNOT_USE_POWER");
        }
        let has = pl.discard.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::LIGHTNING)
        });
        if !has {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(dynamotor(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let mut o = AttachOpts::new(g.st.players[p].discard.len() as u8);
        o.allow_cancel = true;
        o.min = 1;
        o.max = 1;
        let filter = Filter {
            super_type: Some(SuperType::Energy as u8),
            energy_type: Some(EnergyType::Basic as u8),
            name: Some("Lightning Energy"),
            ..Filter::none()
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_TO_BENCH",
            PromptKind::AttachEnergy { cards: ListRef::Discard(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].marker.remove_from(dynamotor(), me);
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
    if transfers.is_empty() {
        return Ok(());
    }
    g.st.players[p].marker.add(dynamotor(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], me)?;
    }
    Ok(())
}
