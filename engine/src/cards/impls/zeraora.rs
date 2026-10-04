//! Zeraora (DRI): Scratch — 20. Thunder Blitz — discard all Energy from this
//! Pokémon; 210 damage to 1 of the opponent's Benched Pokémon ex.
//!
//! Fixed (phase 4b): the "any ex" precondition only counts the opponent's
//! Benched Pokémon (it counted the Active Pokémon too, so the attack could be
//! used with every Benched target blocked and the prompt had no valid answer).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Zeraora@Zeraora DRI",
    mask: mask(&[k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 1, me) {
        return Ok(());
    }
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    let pu = p as usize;
    let o = opp as usize;
    let is_ex = |g: &Game, c: CardId| g.st.cdef(c).has_tag(tag::POKEMON_EX_LOWER);
    let ex_on_bench = g.st.players[o].bench.iter().any(|b| g.st.slot_pokemon(o, *b).map_or(false, |c| is_ex(g, c)));
    if !ex_on_bench {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let active = SlotRef::new(pu, g.st.players[pu].active);
    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p, source: active, energy_map: SVec::new() })?;
    let mut cards: SVec<CardId, 16> = SVec::new();
    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
        for m in energy_map.iter() {
            cards.push(m.card);
        }
    }
    let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: active };
    g.run_fx(Effect::DiscardCards { b, cards })?;

    let mut blocked: TargetList = SVec::new();
    for (_, c, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
        if !is_ex(g, c) {
            blocked.push(t);
        }
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    let id = g.player_id(pu);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let r = (|| -> R {
        for t in targets {
            put_damage(g, atk, 210, t)?;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
