//! N's Plot (SV11B, Twinleaf class NsPlan): move up to 2 Energy from your
//! Benched Pokémon to your Active Pokémon.
//!
//! Fixed in phase 4b (R4): playable only when a Benched Pokémon has Energy
//! (Energy on the Active alone opened a prompt with no valid answer). Twinleaf
//! quirk kept: every transfer goes to the Active whatever destination was
//! chosen.
//!
//! R7C: as the effect of an attack (Mr. Mime's Look-Alike Show) the prompt is min 0
//! (rulings 1844, 1853); played from the hand it is min 1.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NsPlan", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

/// `forEachPokemon` targets: (target, slot) for slots holding a Pokémon.
fn pokemon_targets(g: &Game, p: usize) -> Vec<(CardTarget, SlotId)> {
    let pl = &g.st.players[p];
    let mut out = Vec::new();
    if g.st.slot_pokemon(p, pl.active).is_some() {
        out.push((CardTarget::new(PlayerType::BottomPlayer, SlotType::Active, 0), pl.active));
    }
    for (i, &b) in pl.bench.iter().enumerate() {
        if g.st.slot_pokemon(p, b).is_some() {
            out.push((CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, i as u8), b));
        }
    }
    out
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let targets = pokemon_targets(g, p);
    let has_energy = targets.iter().any(|(t, s)| t.slot == SlotType::Bench && g.st.slot(p, *s).cards.iter().any(|c| g.st.cdef(c).is_energy()));
    if !has_energy || targets.len() <= 1 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);

    let mut o = MoveOpts { allow_cancel: false, min: if trainer_via_attack(g, e) { 0 } else { 1 }, max: Some(2), ..Default::default() };
    for (t, s) in targets.iter() {
        if t.slot == SlotType::Active {
            let mut b = Blocked::default();
            for i in 0..g.st.slot(p, *s).cards.len() {
                b.push(i as u8);
            }
            o.blocked_map.push((*t, b));
        } else {
            o.blocked_to.push(*t);
        }
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    slots.push(SlotType::Active as u8);
    let filter = Filter::super_type(SuperType::Energy);
    let id = g.player_id(p);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    g.prompt(id, "MOVE_ENERGY_CARDS", PromptKind::MoveEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o }, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let transfers = match results.first() {
        Some(Res::Transfers(t)) => *t,
        _ => return Ok(()),
    };
    for (from, _to, card) in transfers.iter() {
        let src = get_target(&g.st, p, *from)?;
        let a = g.st.players[p].active;
        move_cards(g, src.list(), ListRef::Slot(p as u8, a), &[*card], me)?;
    }
    Ok(())
}
