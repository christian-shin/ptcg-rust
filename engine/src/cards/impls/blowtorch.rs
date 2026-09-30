//! Blowtorch (PFL): discard a Basic [R] Energy card from your hand to use
//! this card. Discard a Pokémon Tool or Special Energy card from 1 of your
//! opponent's Pokémon, or discard a Stadium in play.
//!
//! Twinleaf: the energy choice (min 1, name "Fire Energy") is queued without
//! waiting; the SelectPrompt offers Tool / Special Energy / Stadium from the
//! counts taken when the card was played (the Special Energy blocked list is
//! taken after the energy discard). After the chosen action queues its
//! prompt (or discards the Stadium), the card moves supporter→discard.
//! The multi-tool branch chooses from `cardList.cards`, where Tools never
//! are.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Blowtorch", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

const TOOL: &str = "CHOICE_TOOL";
const SPECIAL: &str = "CHOICE_SPECIAL_ENERGY";
const STADIUM: &str = "CHOICE_STADIUM";

/// Option lists by availability mask (bit 0 tool, bit 1 special, bit 2 stadium).
const OPTIONS: [&[&str]; 8] = [&[], &[TOOL], &[SPECIAL], &[TOOL, SPECIAL], &[STADIUM], &[TOOL, STADIUM], &[SPECIAL, STADIUM], &[TOOL, SPECIAL, STADIUM]];

fn has_fire(g: &Game, p: usize) -> bool {
    g.st.players[p].hand.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_energy() && d.name == "Fire Energy"
    })
}

fn has_special(g: &Game, o: usize, s: SlotId) -> bool {
    g.st.slot(o, s).energies.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_energy() && d.energy_type == EnergyType::Special as u8
    })
}

/// Bit 0 = Active, bit 1 + i = Bench i.
fn target_bit(t: &CardTarget) -> u32 {
    if t.slot == SlotType::Active {
        1
    } else {
        1 << (1 + t.index as u32)
    }
}

fn targets_of(mask: u32) -> TargetList {
    let mut v = TargetList::new();
    if mask & 1 != 0 {
        v.push(CardTarget::new(PlayerType::TopPlayer, SlotType::Active, 0));
    }
    for i in 0..8u8 {
        if mask & (1 << (1 + i as u32)) != 0 {
            v.push(CardTarget::new(PlayerType::TopPlayer, SlotType::Bench, i));
        }
    }
    v
}

fn choose_target(g: &mut Game, me: CardId, p: usize, blocked: TargetList, stage: u8) {
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(stage);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DISCARD_CARDS",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    if !has_fire(g, p) {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut with_tool = 0;
    let mut blocked = 0u32;
    let mut special = 0;
    for (s, _, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter() {
        if !g.st.slot(o, *s).tools.is_empty() {
            with_tool += 1;
        } else {
            blocked |= target_bit(t);
        }
        if has_special(g, o, *s) {
            special += 1;
        }
    }
    let stadium = g.st.stadium_card().is_some();
    if with_tool == 0 && !stadium && special == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    let mut avail = 0;
    if with_tool > 0 {
        avail |= 1;
    }
    if special > 0 {
        avail |= 2;
    }
    if stadium {
        avail |= 4;
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = blocked as i32;
    f.a[2] = avail;
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Fire Energy"), ..Filter::none() };
    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), filter, ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
    Ok(())
}

fn discard_me(g: &mut Game, me: CardId, p: usize) -> R {
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let o = 1 - p;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        // Energy discarded: offer the options.
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
            let mut special_blocked = 0u32;
            for (s, _, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter() {
                if !has_special(g, o, *s) {
                    special_blocked |= target_bit(t);
                }
            }
            let mut nf = f;
            nf.stage = 2;
            nf.a[3] = special_blocked as i32;
            let values = OPTIONS[(f.a[2] & 7) as usize];
            let id = g.player_id(p);
            g.prompt(
                id,
                "DISCARD_STADIUM_OR_TOOL_OR_SPECIAL_ENERGY",
                PromptKind::Select { values: SelectValues::Static(values), allow_cancel: false, default_value: 0 },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        // Option chosen.
        2 => {
            let values = OPTIONS[(f.a[2] & 7) as usize];
            let choice = first.as_int();
            if choice < 0 || choice as usize >= values.len() {
                bail!("TypeError: Cannot read properties of undefined (reading 'action')");
            }
            match values[choice as usize] {
                TOOL => choose_target(g, me, p, targets_of(f.a[1] as u32), 3),
                SPECIAL => choose_target(g, me, p, targets_of(f.a[3] as u32), 4),
                _ => {
                    let stadium = match g.st.stadium_card() {
                        Some(s) => s,
                        None => bail!("CANNOT_PLAY_THIS_CARD"),
                    };
                    if let Some(l) = g.st.locate(stadium) {
                        let owner = l.owner().unwrap_or(p);
                        g.run_fx(Effect::MoveCards {
                            source: l,
                            destination: ListRef::Discard(owner as u8),
                            cards: None,
                            count: None,
                            to_top: false,
                            to_bottom: false,
                            skip_cleanup: false,
                            source_card: me,
                        })?;
                    }
                    discard_me(g, me, p)?;
                }
            }
            discard_me(g, me, p)
        }
        // Tool target chosen.
        3 => {
            let t = match first.slots().first() {
                Some(t) => *t,
                None => return Ok(()),
            };
            let (tp, ts) = (t.p as usize, t.s);
            let tools = g.st.slot(tp, ts).tools;
            if !tools.is_empty() {
                if tools.len() > 1 {
                    let mut nf = CardFrame::at(5);
                    nf.a[0] = p as i32;
                    nf.a[1] = tp as i32;
                    nf.a[2] = ts as i32;
                    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Tool as u8), ..Filter::none() };
                    choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Slot(tp as u8, ts), filter, ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: nf });
                    return Ok(());
                }
                let tool = tools.get(0).unwrap();
                move_cards(g, ListRef::Slot(tp as u8, ts), ListRef::Discard(tp as u8), &[tool], me)?;
            }
            discard_me(g, me, p)
        }
        // Special Energy target chosen.
        4 => {
            let t = match first.slots().first() {
                Some(t) => *t,
                None => return Ok(()),
            };
            let mut nf = CardFrame::at(6);
            nf.a[0] = p as i32;
            nf.a[1] = t.p as i32;
            nf.a[2] = t.s as i32;
            let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Special as u8), ..Filter::none() };
            choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Slot(t.p, t.s), filter, ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: nf });
            Ok(())
        }
        // Multi-tool choice.
        5 => {
            let (tp, ts) = (f.a[1] as usize, f.a[2] as SlotId);
            if let Some(&c) = first.cards().first() {
                move_cards(g, ListRef::Slot(tp as u8, ts), ListRef::Discard(tp as u8), &[c], me)?;
            }
            discard_me(g, me, p)
        }
        // Special Energy chosen.
        6 => {
            let (tp, ts) = (f.a[1] as usize, f.a[2] as SlotId);
            let cards: Vec<CardId> = first.cards().to_vec();
            if !cards.is_empty() {
                move_cards(g, ListRef::Slot(tp as u8, ts), ListRef::Discard(o as u8), &cards, me)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
