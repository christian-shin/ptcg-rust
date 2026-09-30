//! Hop's Dubwool (JTG): Defiant Horn — when you play this Pokémon from your
//! hand to evolve 1 of your Pokémon during your turn, you may switch in 1
//! of your opponent's Benched Pokémon to the Active Spot. Headbutt — 80.
//!
//! Twinleaf: the ability-lock probe runs before the ConfirmPrompt;
//! accepting with an empty opposing Bench throws CANNOT_PLAY_THIS_CARD in
//! the callback.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HopsDubwool", mask: mask(&[k::PLAY_POKEMON]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card != me {
            return Ok(());
        }
        let p = p as usize;
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(id, "WANT_TO_USE_ABILITY", PromptKind::Confirm, Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    let p = f.a[0] as usize;
    let o = 1 - p;
    match f.stage {
        1 => {
            if !first.as_bool() {
                return Ok(());
            }
            let pl = &g.st.players[o];
            if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
                bail!("CANNOT_PLAY_THIS_CARD");
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_SWITCH",
                PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
                Cont::Card { card: me, frame: CardFrame { stage: 2, ..f } },
            );
            Ok(())
        }
        2 => {
            let t = match first.slots().first().copied() {
                Some(t) => t,
                None => bail!("TypeError: Cannot read properties of undefined"),
            };
            crate::engine::turn::switch_pokemon(g, o, t.s)?;
            Ok(())
        }
        _ => Ok(()),
    }
}
