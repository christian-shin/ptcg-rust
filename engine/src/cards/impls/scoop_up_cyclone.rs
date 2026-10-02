//! Scoop Up Cyclone (TWM, ACE SPEC): put 1 of your Pokémon and all cards
//! attached to it into your hand.
//!
//! Twinleaf: the Trainer play is prevented (the card is never discarded by
//! the card itself); a non-cancellable ChoosePokemonPrompt over the Active and
//! Bench, then one MOVE_POKEMON_OFF_BOARD to the hand.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ScoopUpCyclone", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    g.set_prevent(e, true);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_PICK_UP",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    if let Some(s) = results.first().and_then(|r| r.slots().first().copied()) {
        move_pokemon_off_board(g, s, ListRef::Hand(p as u8), me)?;
    }
    Ok(())
}
