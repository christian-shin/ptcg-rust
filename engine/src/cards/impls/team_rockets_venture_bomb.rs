//! Team Rocket's Venture Bomb (DRI): flip a coin. If heads, put 2 damage
//! counters on 1 of your opponent's Pokémon. If tails, put 2 damage counters
//! on your Active Pokémon.
//!
//! Twinleaf quirks kept: counters are a raw `damage += 20` (no effects).
//! Right after the coin prompt is created (before it resolves), a
//! MOVE_CARDS supporter→discard with no card list moves the WHOLE play pile
//! (this card plus anything else in it) to the discard.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "TeamRocketsVentureBomb",
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
    let mut f = CardFrame::at(0);
    f.a[0] = p as i32;
    g.coin_flip(p, CoinCb::Card { card: me, frame: f })?;
    g.run_fx(Effect::MoveCards {
        source: ListRef::Supporter(p as u8),
        destination: ListRef::Discard(p as u8),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    let p = f.a[0] as usize;
    if !heads {
        let a = g.st.players[p].active;
        g.st.players[p].slots[a as usize].damage += 20;
        return Ok(());
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut nf = CardFrame::at(1);
    nf.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: nf },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let first = results.first().copied().unwrap_or(Res::Null);
    for t in first.slots().iter() {
        g.st.players[t.p as usize].slots[t.s as usize].damage += 20;
    }
    Ok(())
}
