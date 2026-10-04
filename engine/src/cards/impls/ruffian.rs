//! Ruffian (JTG, supporter): discard a Pokémon Tool and a Special Energy
//! from 1 of your opponent's Pokémon.
//!
//! Fixed (phase 4b #46): a target needs both a Tool and a Special Energy
//! (Twinleaf accepted either); the effect is prevented and the Supporter is
//! moved to the Supporter area first (as Rust Syndicate Grunt does), the prompt
//! message is CHOOSE_POKEMON_TO_DISCARD_CARDS and CLEAN_UP_SUPPORTER discards
//! the Supporter when the prompts finish. Targets without a Special Energy and
//! a Tool are blocked. With one
//! Tool it is discarded without a prompt; with several, a prompt over the
//! whole card list picks one. The Special Energy prompt (min 1) is a
//! ChooseCardsPrompt over the target's full card list.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Ruffian", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let o = 1 - p;
    let mut any = false;
    let mut blocked = TargetList::new();
    for (s, _, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
        let slot = g.st.slot(o, s);
        if slot.energies.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Special as u8
        }) && slot.tools.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_trainer() && d.trainer_type == TrainerType::Tool as u8
        }) {
            any = true;
        } else {
            blocked.push(t);
        }
    }
    if !any {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DISCARD_CARDS",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

/// `CLEAN_UP_SUPPORTER` (standard format): `supporter.moveCardTo(card, discard)`.
fn supporter_to_discard(g: &mut Game, me: CardId, p: usize) -> R {
    g.move_card_to(ListRef::Supporter(p as u8), me, ListRef::Discard(p as u8));
    Ok(())
}

/// The "removing special energies" block.
fn energy_part(g: &mut Game, me: CardId, p: usize, t: SlotRef) -> R {
    let n = g.st.slot(t.p as usize, t.s).energies.iter().filter(|c| {
        let d = g.st.cdef(*c);
        d.is_energy() && d.energy_type == EnergyType::Special as u8
    }).count();
    if n > 0 {
        let mut f = CardFrame::at(3);
        f.a[0] = p as i32;
        f.l[0] = t.p;
        f.l[1] = t.s;
        let mut filter = Filter::none();
        filter.super_type = Some(SuperType::Energy as u8);
        filter.energy_type = Some(EnergyType::Special as u8);
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_CARD_TO_DISCARD",
            PromptKind::ChooseCards { cards: t.list(), filter, opts: ChooseCardsOpts::new(1, 1, false) },
            Cont::Card { card: me, frame: f },
        );
        Ok(())
    } else {
        supporter_to_discard(g, me, p)
    }
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let o = 1 - p;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let t = match first.slots().first() {
                Some(t) => *t,
                None => return supporter_to_discard(g, me, p),
            };
            let tools: Vec<CardId> = g.st.slot(t.p as usize, t.s).tools.iter().collect();
            if !tools.is_empty() {
                if tools.len() > 1 {
                    let mut nf = CardFrame::at(2);
                    nf.a[0] = p as i32;
                    nf.l[0] = t.p;
                    nf.l[1] = t.s;
                    let mut filter = Filter::none();
                    filter.super_type = Some(SuperType::Trainer as u8);
                    filter.trainer_type = Some(TrainerType::Tool as u8);
                    let id = g.player_id(p);
                    g.prompt(
                        id,
                        "CHOOSE_CARD_TO_DISCARD",
                        PromptKind::ChooseCards { cards: t.list(), filter, opts: ChooseCardsOpts::new(1, 1, false) },
                        Cont::Card { card: me, frame: nf },
                    );
                    return Ok(());
                }
                move_cards(g, t.list(), ListRef::Discard(o as u8), &[tools[0]], me)?;
            }
            energy_part(g, me, p, t)
        }
        2 => {
            let t = SlotRef::new(f.l[0] as usize, f.l[1]);
            let sel: Vec<CardId> = first.cards().to_vec();
            if let Some(c) = sel.first() {
                move_cards(g, t.list(), ListRef::Discard(o as u8), &[*c], me)?;
            }
            energy_part(g, me, p, t)
        }
        3 => {
            let t = SlotRef::new(f.l[0] as usize, f.l[1]);
            let sel: Vec<CardId> = first.cards().to_vec();
            move_cards(g, t.list(), ListRef::Discard(o as u8), &sel, me)?;
            supporter_to_discard(g, me, p)
        }
        _ => Ok(()),
    }
}
