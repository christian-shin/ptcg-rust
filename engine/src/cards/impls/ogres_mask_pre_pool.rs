//! Ogre's Mask (PRE 118, Item): choose a Pokémon ex in your discard pile that
//! has "Ogerpon" in its name, and switch it with 1 of your Pokémon ex in play
//! that has "Ogerpon" in its name. Attached cards, damage counters, Special
//! Conditions, turns in play and other effects remain on the new Pokémon.
//!
//! Twinleaf: throws CANNOT_PLAY_THIS_CARD when no discard card (or no
//! in-play top Pokémon) is an Ogerpon ex; the card is moved to the
//! supporter zone by hand (preventDefault). The discard Pokémon is chosen
//! first (non-Ogerpon-ex cards blocked), then the in-play Pokémon. The new
//! card goes in first, the old one is discarded, and the new card is
//! re-inserted at the old card's index in `slot.cards`. Finally the Mask is
//! moved supporter -> discard.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "OgresMaskPREPool", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn is_ogerpon_ex(g: &Game, c: CardId) -> bool {
    let d = g.st.cdef(c);
    d.is_pokemon() && d.has_tag(tag::POKEMON_EX_LOWER) && d.name.contains("Ogerpon")
}

fn blocked_in_play(g: &Game, p: usize) -> (bool, TargetList) {
    let mut has = false;
    let mut blocked = TargetList::new();
    for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
        if is_ogerpon_ex(g, *c) {
            has = true;
        } else {
            blocked.push(*t);
        }
    }
    (has, blocked)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let pu = p as u8;
    let mut opts = ChooseCardsOpts::new(1, 1, false);
    let mut blocked_discard = 0usize;
    for (i, c) in g.st.players[p].discard.iter().enumerate() {
        if !is_ogerpon_ex(g, c) {
            blocked_discard += 1;
            opts.blocked.push(i as u8);
        }
    }
    let (has_in_play, _) = blocked_in_play(g, p);
    if blocked_discard == g.st.players[p].discard.len() || !has_in_play {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(pu), ListRef::Supporter(pu), &[me], me)?;
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Discard(pu), Filter::super_type(SuperType::Pokemon), opts, Cont::Card { card: me, frame: f });
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let pu = p as u8;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let new_card = match first.cards().first() {
                Some(c) if is_ogerpon_ex(g, *c) => *c,
                _ => bail!("INVALID_PROMPT_RESULT"),
            };
            let (_, blocked) = blocked_in_play(g, p);
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.a[1] = new_card as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_SWITCH",
                PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let new_card = f.a[1] as CardId;
            let t = match first.slots().first() {
                Some(t) => *t,
                None => bail!("INVALID_PROMPT_RESULT"),
            };
            let (tp, ts) = (t.p as usize, t.s);
            let old_card = match g.st.slot_pokemon(tp, ts) {
                Some(c) if is_ogerpon_ex(g, c) => c,
                _ => bail!("INVALID_PROMPT_RESULT"),
            };
            let list = ListRef::Slot(tp as u8, ts);
            let old_index = g.st.slot(tp, ts).cards.index_of(old_card);
            move_cards(g, ListRef::Discard(pu), list, &[new_card], me)?;
            move_cards(g, list, ListRef::Discard(pu), &[old_card], me)?;
            let slot = &mut g.st.players[tp].slots[ts as usize];
            let new_index = slot.cards.index_of(new_card);
            if let (Some(ni), Some(oi)) = (new_index, old_index) {
                if ni != oi {
                    slot.cards.remove_at(ni);
                    let at = oi.min(slot.cards.len());
                    slot.cards.insert(at, new_card);
                }
            }
            // It is the same Pokémon (ruling 1840): the state kept on the card object moves to the new card.
            transfer_pokemon_card_state(g, p, old_card, new_card);
            move_cards(g, ListRef::Supporter(pu), ListRef::Discard(pu), &[me], me)?;
            Ok(())
        }
        _ => Ok(()),
    }
}
