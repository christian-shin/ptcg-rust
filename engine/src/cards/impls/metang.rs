//! Metang (TEF): Metal Maker — once during your turn, look at the top 4 cards
//! of your deck and attach any number of [M] Energy you find there to your
//! Pokémon; put the rest on the bottom of your deck.
//!
//! LOOK_AT_TOP_X_CARDS_AND_ATTACH_UP_TO_Y_ENERGY(4, 4, { energyFilter: Basic,
//! validCardTypes: [M], remainderDestination: 'bottom' }). Phase 4b (R6): the
//! text says Basic [M] Energy, so the prompt filter (and the max, which counts
//! the top cards matching the energy filter) is Basic Energy only; Special
//! Energy that provides [M] (Magnetic Metal Energy) used to be attachable.
//! Twinleaf quirks kept: the prompt max counts every Basic Energy card among
//! the top cards (not only [M]); the remainder is not shuffled; the marker is
//! set even when the deck is empty.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Metang",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::POWER]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn metal_maker() -> crate::markers::MarkerName {
    crate::marker!("METAL_MAKER_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(metal_maker(), me);
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(metal_maker(), me) {
            m.remove_from(metal_maker(), me);
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(metal_maker(), me) {
            bail!("POWER_ALREADY_USED");
        }
        g.st.players[p].marker.add(metal_maker(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        ability_used(g, p, me);

        let deck_len = g.st.players[p].deck.len();
        if deck_len == 0 {
            return Ok(());
        }
        let top = g.alloc_temp(&[]);
        g.move_to(ListRef::Deck(p as u8), top, Some(4.min(deck_len)));
        let energies = g.lst(top).iter().filter(|c| g.st.cdef(**c).is_energy() && g.st.cdef(**c).energy_type == EnergyType::Basic as u8).count();
        let max_attach = 4.min(energies) as u8;
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        slots.push(SlotType::Active as u8);
        let mut o = AttachOpts::new(max_attach);
        o.allow_cancel = false;
        o.min = 0;
        o.max = max_attach;
        let mut vt = SVec::new();
        vt.push(ct::METAL);
        o.valid_card_types = Some(vt);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.l[0] = match top {
            ListRef::Temp(i) => i,
            _ => unreachable!(),
        };
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_CARDS",
            PromptKind::AttachEnergy {
                cards: top,
                player_type: PlayerType::BottomPlayer,
                slots,
                filter: Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Default::default() },
                o,
            },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let top = ListRef::Temp(f.l[0]);
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    // maxPokemonTargets = 4: more distinct targets than 4 is impossible
    // (at most 4 transfers).
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        g.run_fx(Effect::AttachEnergy { p: p as u8, card: c, target })?;
    }
    // moveRemainingTopDeckCards(..., 'bottom'): push without shuffling.
    let rest: Vec<CardId> = g.lst(top).to_vec();
    for c in rest {
        g.st.players[p].deck.push(c);
    }
    g.lst_mut(top).clear();
    Ok(())
}
