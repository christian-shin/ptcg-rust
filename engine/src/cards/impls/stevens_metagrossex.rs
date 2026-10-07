//! Steven's Metagross ex (DRI / ASC): X-Boot - once during your turn, search
//! your deck for a Basic [P] Energy, a Basic [M] Energy, or 1 of each and
//! attach them to your [P] and [M] Pokémon in any way, then shuffle. Metal
//! Stomp - 200.
//!
//! Fixed (phase 4b): AttachEnergyPrompt's validate no longer returns early on
//! validCardTypes, so differentTypes applies here: two Energy of the same
//! type can't be picked (the same-name throw in the callback, which that
//! pair used to reach, is removed).
//!
//! Fixed (phase 4b, R3): the Energy can go only to [P] and [M] Pokémon: the
//! AttachEnergyPrompt's blockedTo lists every other Pokémon (CheckPokemonType
//! on each slot; it used to offer them all).
//!
//! Twinleaf quirks kept: the marker (X_BOOT_MARKER, source this card) and
//! ABILITY_USED are set *before* the prompt; the AttachEnergyPrompt (deck ->
//! Bench + Active, basic Energy, min 0, max 2, differentTypes, validCardTypes
//! [P, M], cancellable) is answered in the callback; each transfer is a
//! MOVE_CARDS (no AttachEnergyEffect), then SHUFFLE_DECK. No deck check, so
//! an empty deck still uses the Ability. The marker is cleared at the end of
//! the turn and on this card's PlayPokemonEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "StevensMetagrossex",
    mask: mask(&[k::POWER, k::END_TURN, k::PLAY_POKEMON]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn x_boot() -> crate::markers::MarkerName {
    crate::marker!("X_BOOT_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(x_boot(), me) {
            bail!("POWER_ALREADY_USED");
        }
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        ability_used(g, p, me);
        g.st.players[p].marker.add(x_boot(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        slots.push(SlotType::Active as u8);
        let mut o = AttachOpts::new(g.st.players[p].deck.len().min(255) as u8);
        o.allow_cancel = true;
        o.min = 0;
        o.max = 2;
        o.different_types = true;
        let mut vt = SVec::new();
        vt.push(ct::PSYCHIC);
        vt.push(ct::METAL);
        o.valid_card_types = Some(vt);
        // Only [P] Pokémon and [M] Pokémon can receive the Energy.
        for (slot, _c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            let sr = SlotRef::new(p, slot);
            let types = crate::engine::game_effect::pokemon_types(g, sr);
            let (te, _) = g.run_fx(Effect::CheckPokemonType { target: sr, card_types: types })?;
            let ok = matches!(te, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::PSYCHIC) || card_types.contains(&ct::METAL));
            if !ok {
                o.blocked_to.push(t);
            }
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_CARDS",
            PromptKind::AttachEnergy {
                cards: ListRef::Deck(p as u8),
                player_type: PlayerType::BottomPlayer,
                slots,
                filter: Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() },
                o,
            },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    remove_marker_at_end_of_turn(g, e, x_boot(), me);
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(x_boot(), me);
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
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Deck(p as u8), ListRef::Slot(target.p, target.s), &[c], me)?;
    }
    shuffle_deck(g, p);
    Ok(())
}
