//! Rosa's Encouragement (POR / M3): only if you have more Prize cards
//! remaining than your opponent; attach up to 2 Basic Energy cards from your
//! discard pile to 1 of your Stage 2 Pokémon.
//!
//! Fixed in phase 4b: the prompt filter also had `stage: STAGE_2`, which no
//! Energy card matches, so no card was ever selectable and nothing attached.
//! The filter is now Basic Energy only and the non-Stage 2 Pokémon are in
//! `blockedTo` (the prompt's validate checks neither).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "RosasEncouragement", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    let played_from_hand = !trainer_via_attack(g, e);
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    if g.st.players[p].prize_left() <= g.st.players[o].prize_left() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let basic = g
        .st
        .players[p]
        .discard
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        })
        .count();
    if basic == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let pl = &g.st.players[p];
    let mut slots_all: Vec<SlotId> = vec![pl.active];
    slots_all.extend(pl.bench.iter().copied());
    let stage2 = slots_all.iter().any(|s| g.st.slot_pokemon(p, *s).map(|c| g.st.cdef(c).stage == Stage::Stage2 as u8).unwrap_or(false));
    if !stage2 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let max = basic.min(2) as u8;
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut opts = AttachOpts::new(max);
    opts.allow_cancel = false;
    // Fixed (phase 4b, rulings 1778/1853): "up to 2" from a public zone takes at least 1 when played from the hand;
    // used through an attack (Look-Alike Show) it may be 0 (ruling 1844).
    opts.min = if played_from_hand { 1 } else { 0 };
    opts.max = max;
    opts.same_target = true;
    for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.cdef(c).stage != Stage::Stage2 as u8 {
            opts.blocked_to.push(t);
        }
    }
    let filter = Filter {
        super_type: Some(SuperType::Energy as u8),
        energy_type: Some(EnergyType::Basic as u8),
        ..Filter::none()
    };
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_CARDS",
        PromptKind::AttachEnergy { cards: ListRef::Discard(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o: opts },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    if let Some(Res::Attach(ts)) = results.first() {
        for (to, c) in ts.iter() {
            let target = get_target(&g.st, p, *to)?;
            move_cards(g, ListRef::Discard(p as u8), target.list(), &[*c], me)?;
        }
    }
    Ok(())
}

