//! Crushing Hammer (SVI): flip a coin; if heads, discard an Energy attached
//! to 1 of your opponent's Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "CrushingHammer",
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
    let mut has = false;
    let mut blocked: SVec<CardTarget, 8> = SVec::new();
    // opponent.forEachPokemon(TOP_PLAYER, ...): slots holding a Pokémon.
    let mut all: Vec<(CardTarget, SlotId)> = vec![(CardTarget::new(PlayerType::TopPlayer, SlotType::Active, 0), g.st.players[o].active)];
    for (i, b) in g.st.players[o].bench.iter().enumerate() {
        all.push((CardTarget::new(PlayerType::TopPlayer, SlotType::Bench, i as u8), *b));
    }
    for (t, s) in all {
        if g.st.slot_pokemon(o, s).is_none() {
            continue;
        }
        if !g.st.slot(o, s).energies.is_empty() {
            has = true;
        } else {
            blocked.push(t);
        }
    }
    if !has {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut f = CardFrame::at(0);
    f.a[0] = p as i32;
    // Blocked targets travel in the frame: (slot, index) packed per entry.
    f.l[0] = blocked.len() as u8;
    let mut packed: u32 = 0;
    for (i, t) in blocked.iter().enumerate() {
        let code = if t.slot == SlotType::Active { 0 } else { 1 + t.index as u32 };
        packed |= code << (4 * i);
    }
    f.a[1] = packed as i32;
    g.coin_flip(p, CoinCb::Card { card: me, frame: f })?;
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    if !heads {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let mut blocked: TargetList = SVec::new();
    let packed = f.a[1] as u32;
    for i in 0..f.l[0] as usize {
        let code = (packed >> (4 * i)) & 0xf;
        let t = if code == 0 {
            CardTarget::new(PlayerType::TopPlayer, SlotType::Active, 0)
        } else {
            CardTarget::new(PlayerType::TopPlayer, SlotType::Bench, (code - 1) as u8)
        };
        blocked.push(t);
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut nf = CardFrame::at(1);
    nf.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DISCARD_CARDS",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: nf },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let t = match first.slots().first() {
                Some(t) => *t,
                None => return Ok(()),
            };
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.l[0] = t.p;
            nf.l[1] = t.s;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_CARD_TO_DISCARD",
                PromptKind::ChooseCards {
                    cards: ListRef::SlotEnergies(t.p, t.s),
                    filter: Filter::none(),
                    opts: ChooseCardsOpts::new(1, 1, false),
                },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let (tp, ts) = (f.l[0], f.l[1]);
            let cards: Vec<CardId> = first.cards().to_vec();
            let o = 1 - p;
            // `cards = selected` (never null: allowCancel is false).
            move_cards(g, ListRef::Slot(tp, ts), ListRef::Discard(o as u8), &cards, me)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

