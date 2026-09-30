//! Enhanced Hammer (TWM): discard a Special Energy attached to 1 of your
//! opponent's Pokémon.
//!
//! Twinleaf: Pokémon without a Special Energy in `energies` are blocked; the
//! card choice is on the target's `energies` list (energyType SPECIAL, min 1,
//! no cancel) and the energy moves from the slot to the opponent's discard.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EnhancedHammer", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    let mut has = false;
    let mut blocked: TargetList = SVec::new();
    for (s, _, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
        if g.st.slot(o, s).energies.iter().any(|c| g.st.cdef(c).energy_type == EnergyType::Special as u8) {
            has = true;
        } else {
            blocked.push(t);
        }
    }
    if !has {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
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

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let t = match first.slots().first() {
                Some(t) => *t,
                None => return Ok(()),
            };
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.l[0] = t.p;
            nf.l[1] = t.s;
            let mut filter = Filter::none();
            filter.energy_type = Some(EnergyType::Special as u8);
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_CARD_TO_DISCARD",
                PromptKind::ChooseCards { cards: ListRef::SlotEnergies(t.p, t.s), filter, opts: ChooseCardsOpts::new(1, 1, false) },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if !cards.is_empty() {
                let o = 1 - p;
                move_cards(g, ListRef::Slot(f.l[0], f.l[1]), ListRef::Discard(o as u8), &cards, me)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
