//! Jacinthe (POR, supporter): heal 150 damage from 1 of your [P] Pokémon.
//!
//! Twinleaf: the supporter and "[P] Pokémon with damage" checks use a
//! CheckPokemonTypeEffect per Pokémon (no `canPlay` quirks matter). The card
//! moves to the supporter pile and the trainer effect is prevented before the
//! (uncancellable, unblocked) prompt: any of your Pokémon can be chosen, and
//! the heal only happens when the chosen one is [P] at resolution.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Jacinthe", mask: mask(&[k::TRAINER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn is_psychic(g: &mut Game, s: SlotRef) -> Result<bool, GameError> {
    let types = crate::engine::game_effect::pokemon_types(g, s);
    let (e, _) = g.run_fx(Effect::CheckPokemonType { target: s, card_types: types })?;
    Ok(matches!(e, Effect::CheckPokemonType { card_types, .. } if card_types.contains(&ct::PSYCHIC)))
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let mut psychic: SVec<SlotId, 9> = SVec::new();
    for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
        if is_psychic(g, SlotRef::new(p, *s))? {
            psychic.push(*s);
        }
    }
    if psychic.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if !psychic.iter().any(|s| g.st.slot(p, *s).damage > 0) {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_HEAL",
        PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    if let Some(t) = results.first().and_then(|r| r.slots().first().copied()) {
        if is_psychic(g, t)? {
            g.run_fx(Effect::Heal { p: p as u8, target: t, damage: 150 })?;
        }
    }
    Ok(())
}
