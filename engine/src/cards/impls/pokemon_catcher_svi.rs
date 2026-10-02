//! Pokémon Catcher (SVI, as Pokemon Catcher POR): flip a coin; if heads,
//! switch 1 of your opponent's Benched Pokémon with their Active Pokémon.
//!
//! Twinleaf (scarlet-and-violet file): as the SSH port, but the
//! TrainerEffect is marked `preventDefault` after the bench check.
use crate::cards::prelude::*;
use crate::engine::turn::switch_pokemon_silent;

pub static IMPL: CardImpl = CardImpl {
    class: "PokemonCatcher@POR",
    mask: mask(&[k::TRAINER]),
    reduce,
    resume: Some(resume),
    coin: Some(coin),
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    let pl = &g.st.players[o];
    if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut f = CardFrame::at(0);
    f.a[0] = p as i32;
    g.coin_flip(p, CoinCb::Card { card: me, frame: f })?;
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    if !heads {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let mut nf = CardFrame::at(1);
    nf.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_SWITCH",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: nf },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let o = 1 - f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    if let Some(t) = first.slots().first().copied() {
        switch_pokemon_silent(g, o, t.s)?;
    }
    Ok(())
}
