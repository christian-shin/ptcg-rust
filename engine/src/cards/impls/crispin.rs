//! Crispin (SCR): search your deck for up to 2 Basic Energy cards of
//! different types, reveal them, put 1 into your hand and attach the other to
//! 1 of your Pokémon, then shuffle.
//!
//! Fixed (phase 4b): the search prompt is `differentTypes` (a duplicate type
//! can't be picked any more, so the old CAN_ONLY_SELECT_TWO_DIFFERENT_ENERGY_TYPES
//! throw in its callback is gone).
//!
//! Twinleaf order kept: the ShowCards, AttachEnergy and ShuffleDeck prompts
//! are all created by the search callback (the shuffle before the attach is
//! answered); the final shuffle has no trailing WaitPrompt; the ShowCards
//! prompt is created even when nothing was selected.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Crispin", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn basic_energy() -> Filter {
    Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let mut opts = ChooseCardsOpts::new(0, 2, false);
    opts.different_types = true;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), basic_energy(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            let oid = g.player_id(1 - p);
            g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
            let temp = g.alloc_temp(&[]);
            move_cards(g, ListRef::Deck(p as u8), temp, &cards, me)?;
            let ti = match temp {
                ListRef::Temp(i) => i,
                _ => unreachable!(),
            };
            if g.lst(temp).len() == 2 {
                let mut slots = SVec::new();
                slots.push(SlotType::Bench as u8);
                slots.push(SlotType::Active as u8);
                let mut o = AttachOpts::new(2);
                o.allow_cancel = false;
                o.min = 1;
                o.max = 1;
                o.different_targets = true;
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                nf.l[0] = ti;
                let id = g.player_id(p);
                g.prompt(
                    id,
                    "ATTACH_ENERGY_CARDS",
                    PromptKind::AttachEnergy { cards: temp, player_type: PlayerType::BottomPlayer, slots, filter: basic_energy(), o },
                    Cont::Card { card: me, frame: nf },
                );
            }
            if g.lst(temp).len() == 1 {
                let c = g.lst(temp)[0];
                move_cards(g, temp, ListRef::Hand(p as u8), &[c], me)?;
            }
            let mut nf = CardFrame::at(3);
            nf.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let temp = ListRef::Temp(f.l[0]);
            if let Res::Attach(ts) = first {
                for (to, c) in ts.iter() {
                    let target = get_target(&g.st, p, *to)?;
                    move_cards(g, temp, target.list(), &[*c], me)?;
                }
            }
            match g.lst(temp).first().copied() {
                Some(c) => move_cards(g, temp, ListRef::Hand(p as u8), &[c], me)?,
                // `cards: [undefined]`: moveCardsTo finds nothing to move.
                None => move_cards(g, temp, ListRef::Hand(p as u8), &[], me)?,
            }
            Ok(())
        }
        3 => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

