//! Max Rod (PRE, ACE SPEC): choose up to 5 in any combination of Pokémon and
//! Basic Energy cards from your discard pile and put them into your hand.
//!
//! Twinleaf: other cards are blocked; the choice is min 1 / max 5 with no
//! cancel on the (sorted) discard; the reveal comes before MOVE_CARDS.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MaxRod", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut n = 0;
    let mut blocked = Blocked::default();
    for (i, c) in g.st.players[p].discard.iter().enumerate() {
        let d = g.st.cdef(c);
        if d.is_pokemon() || (d.is_energy() && d.energy_type == EnergyType::Basic as u8) {
            n += 1;
        } else {
            blocked.push(i as u8);
        }
    }
    if n == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut opts = ChooseCardsOpts::new(1, 5, false);
    opts.blocked = blocked;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Discard(p as u8), Filter::none(), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if !cards.is_empty() {
                let temp = g.alloc_temp(&cards);
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                nf.l[0] = match temp {
                    ListRef::Temp(i) => i,
                    _ => unreachable!(),
                };
                let id = g.player_id(1 - p);
                g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
                return Ok(());
            }
            move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), &[], me)
        }
        2 => {
            let cards: Vec<CardId> = g.lst(ListRef::Temp(f.l[0])).to_vec();
            move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), &cards, me)
        }
        _ => Ok(()),
    }
}
