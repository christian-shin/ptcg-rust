//! Rust Syndicate Grunt (PBL, supporter): only if any of your Pokémon were
//! Knocked Out during your opponent's last turn; discard an Energy from 1 of
//! your opponent's Pokémon.
//!
//! Fixed (phase 4b #38): Twinleaf also required an otherwise empty hand,
//! which the card text does not say; that check is gone. The effect is
//! prevented after the checks, then the card moves to the supporter pile; the
//! Supporter is discarded by CLEAN_UP_SUPPORTER when the prompts finish.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "RustSyndicateGrunt", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    if !g.st.players[p].pokemon_knocked_out_during_opponents_last_turn {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let o = 1 - p;
    let mut any = false;
    for (s, _, _) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter() {
        if !g.st.slot(o, *s).energies.is_empty() {
            any = true;
        }
    }
    if !any {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    let mut blocked = TargetList::new();
    for (s, _, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter() {
        if g.st.slot(o, *s).energies.is_empty() {
            blocked.push(*t);
        }
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
fn clean_up(g: &mut Game, me: CardId, p: usize) {
    g.move_card_to(ListRef::Supporter(p as u8), me, ListRef::Discard(p as u8));
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let t = match first.slots().first() {
                Some(t) => *t,
                None => {
                    clean_up(g, me, p);
                    return Ok(());
                }
            };
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.l[0] = t.p;
            nf.l[1] = t.s;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_CARD_TO_DISCARD",
                PromptKind::ChooseCards { cards: ListRef::SlotEnergies(t.p, t.s), filter: Filter::none(), opts: ChooseCardsOpts::new(1, 1, false) },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let t = SlotRef::new(f.l[0] as usize, f.l[1]);
            let sel: Vec<CardId> = first.cards().to_vec();
            move_cards(g, t.list(), ListRef::Discard(t.p), &sel, me)?;
            clean_up(g, me, p);
            Ok(())
        }
        _ => Ok(()),
    }
}
