//! Transformation Tome / Book of Transformation (CRI): play 2 at once.
//! Choose a Basic Pokémon in your discard pile and switch it with 1 of your
//! Basic Pokémon in play (attachments, damage and effects stay).
//!
//! Twinleaf: "Basic in play" is a slot whose top card is Basic (phase 4b
//! fix: it used to count a Basic under an evolution, so with only evolved
//! Pokémon in play the card was playable but every slot was blocked and the
//! prompt unanswerable); a Fossil in play (a Trainer card played as a Basic
//! Pokémon) counts, and the in-play choice blocks slots whose top card isn't
//! Basic. The swap MOVE_CARDS the slot's
//! bottom card to the discard first, which empties the slot of Pokémon, so
//! the core discards its attachments and resets it; the discard Basic then
//! goes into the empty slot and the second copy is discarded from hand.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TransformationTome", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

/// `c instanceof PokemonCard && c.stage === Stage.BASIC` (the discard check).
fn is_basic_mon(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    d.is_pokemon() && d.stage == Stage::Basic as u8
}

/// `card.stage === Stage.BASIC` for the top card of a slot in play: no
/// `instanceof` check, so a Fossil (a Trainer played as a Basic Pokémon) counts.
fn is_basic_in_play(g: &Game, c: CardId) -> bool {
    g.st.cdef(c).stage == Stage::Basic as u8
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let name = g.st.cdef(me).name;
    let second = match g.st.players[p].hand.iter().find(|c| *c != me && g.st.cdef(*c).name == name) {
        Some(c) => c,
        None => bail!("CANNOT_PLAY_THIS_CARD"),
    };
    let pl = &g.st.players[p];
    let in_play = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| is_basic_in_play(g, *c));
    let in_discard = pl.discard.iter().any(|c| is_basic_mon(g, c));
    if !in_play || !in_discard {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    let mut blocked = TargetList::new();
    for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
        if g.st.cdef(*c).stage != Stage::Basic as u8 {
            blocked.push(*t);
        }
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    f.a[1] = second as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_SWITCH",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let pu = p as u8;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let t = match first.slots().first() {
                Some(t) => *t,
                None => return Ok(()),
            };
            let mut nf = CardFrame::at(2);
            nf.a[0] = f.a[0];
            nf.a[1] = f.a[1];
            nf.a[2] = t.p as i32;
            nf.a[3] = t.s as i32;
            let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
            choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Discard(pu), filter, ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let chosen = match first.cards().first() {
                Some(c) => *c,
                None => return Ok(()),
            };
            let (tp, ts) = (f.a[2] as usize, f.a[3] as SlotId);
            let list = ListRef::Slot(tp as u8, ts);
            if g.st.slot_pokemon(tp, ts).is_some() {
                if let Some(bottom) = g.st.slot(tp, ts).cards.get(0) {
                    move_cards(g, list, ListRef::Discard(pu), &[bottom], me)?;
                }
            }
            move_cards(g, ListRef::Discard(pu), list, &[chosen], me)?;
            move_cards(g, ListRef::Hand(pu), ListRef::Discard(pu), &[f.a[1] as CardId], me)?;
            Ok(())
        }
        _ => Ok(()),
    }
}
