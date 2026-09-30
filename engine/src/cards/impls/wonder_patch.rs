//! Wondrous Patch (Wonder Patch MBD / PFL): attach a basic [P] Energy card
//! from your discard pile to 1 of your Benched [P] Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "WonderPatch",
    mask: mask(&[k::TRAINER]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let has_energy = g.st.players[p].discard.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::PSYCHIC)
    });
    if !has_energy {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut has_psychic = false;
    let mut blocked_to: SVec<CardTarget, 9> = SVec::new();
    let bench: Vec<SlotId> = g.st.players[p].bench.iter().copied().collect();
    for (index, s) in bench.into_iter().enumerate() {
        if g.st.slot(p, s).cards.is_empty() {
            continue;
        }
        let target = SlotRef::new(p, s);
        let types = crate::engine::game_effect::pokemon_types(g, target);
        let (t, _) = g.run_fx(Effect::CheckPokemonType { target, card_types: types })?;
        let psychic = matches!(t, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::PSYCHIC));
        if psychic {
            has_psychic = true;
        } else {
            blocked_to.push(CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, index as u8));
        }
    }
    if !has_psychic {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let filter = Filter {
        super_type: Some(SuperType::Energy as u8),
        energy_type: Some(EnergyType::Basic as u8),
        name: Some("Psychic Energy"),
        ..Filter::none()
    };
    let n = g.st.players[p].discard.len() as u8;
    let mut o = AttachOpts::new(n);
    o.allow_cancel = false;
    o.min = 1;
    o.max = 1;
    o.blocked_to = blocked_to;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_TO_BENCH",
        PromptKind::AttachEnergy { cards: ListRef::Discard(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => return Ok(()),
    };
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], me)?;
    }
    Ok(())
}
