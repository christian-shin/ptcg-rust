//! Minccino (TEF): Beat — 10. Cleaning Up — discard up to 2 Pokémon Tools
//! from your opponent's Pokémon.
//!
//! Twinleaf has two `Minccino` classes; this port is bound to TEF.
//! The Cleaning Up code is on attack 1 (fixed in phase 4b: it was attached to
//! attack 0, Beat). Cleaning Up does nothing when no opposing Pokémon has a
//! Tool (fixed in R1-12: it used to throw CANNOT_PLAY_THIS_CARD, so the attack
//! was not offered), otherwise the AttackEffect is prevented (no damage; the
//! attack has none) and 1–2 opposing Pokémon with Tools are chosen
//! (cancellable). A target with several Tools gets a non-yielding
//! ChooseCardsPrompt (1–2 Tools); one with a single Tool loses it at once.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Minccino@TEF", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 1, me) {
        return Ok(());
    }
    let (p, opp, source) = match *g.e(e) {
        Effect::Attack { p, opp, source, .. } => (p as usize, opp as usize, source),
        _ => return Ok(()),
    };
    let mut with_tool = 0u8;
    let mut blocked = SVec::new();
    for (s, _, t) in for_each_pokemon(g, opp, PlayerType::TopPlayer).iter().copied() {
        if !g.st.slot(opp, s).tools.is_empty() {
            with_tool += 1;
        } else {
            blocked.push(t);
        }
    }
    if with_tool == 0 {
        return Ok(());
    }
    g.set_prevent(e, true);
    let max = with_tool.min(2);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.l[0] = source.p;
    f.l[1] = source.s;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DISCARD_CARDS",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max, allow_cancel: true, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn source_card(g: &Game, f: &CardFrame) -> CardId {
    g.st.slot_pokemon(f.l[0] as usize, f.l[1]).unwrap_or(NO_CARD)
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let p = f.a[0] as usize;
            let targets: SVec<SlotRef, 8> = {
                let mut v = SVec::new();
                for t in first.slots() {
                    v.push(*t);
                }
                v
            };
            for t in targets.iter().copied() {
                let owner = t.p as usize;
                let tools = g.st.slot(owner, t.s).tools;
                if tools.len() > 1 {
                    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Tool as u8), ..Filter::none() };
                    let mut nf = f;
                    nf.stage = 2;
                    nf.a[2] = owner as i32;
                    nf.a[3] = t.s as i32;
                    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Slot(owner as u8, t.s), filter, ChooseCardsOpts::new(1, 2, false), Cont::Card { card: me, frame: nf });
                } else if let Some(tool) = tools.get(0) {
                    let sc = source_card(g, &f);
                    move_cards(g, ListRef::Slot(owner as u8, t.s), ListRef::Discard(owner as u8), &[tool], sc)?;
                }
            }
            Ok(())
        }
        2 => {
            let owner = f.a[2] as u8;
            let s = f.a[3] as SlotId;
            let selected: Vec<CardId> = first.cards().to_vec();
            if !selected.is_empty() {
                let sc = source_card(g, &f);
                move_cards(g, ListRef::Slot(owner, s), ListRef::Discard(owner), &selected, sc)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
