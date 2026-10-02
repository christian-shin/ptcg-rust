//! Hariyama (M1L): Sumo Catcher — when you play this card from your hand to
//! evolve, you may switch 1 of your opponent's Benched Pokémon with their
//! Active Pokémon. Wild Press — 210; this Pokémon does 70 damage to itself.
//!
//! Twinleaf: fires on any EvolveEffect for this card (Rare Candy included);
//! the switch goes through an EffectOfAbilityEffect and happens only if its
//! target survives.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Hariyama", mask: mask(&[k::EVOLVE, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::Evolve { p, card, .. } = *g.e(e) {
        if card == me && !is_ability_blocked(g, p as usize, me, None) {
            let p = p as usize;
            let o = 1 - p;
            let pl = &g.st.players[o];
            if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
                return Ok(());
            }
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
        }
    }
    if was_attack_used(g, e, 0, me) {
        super::tapu_bulu::this_pokemon_does_damage_to_itself(g, e, 70)?;
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if !first.as_bool() {
                return Ok(());
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Bench as u8);
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON_TO_SWITCH",
                PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let t = match first {
                Res::Slots(s) if !s.is_empty() => s.as_slice()[0],
                _ => bail!("TypeError: Cannot read properties of null"),
            };
            let (fx, _) = g.run_fx(Effect::EffectOfAbility { p: p as u8, power: PowerRef { card: me, index: 0 }, card: me, target: Some(t) })?;
            if let Effect::EffectOfAbility { target: Some(_), .. } = fx {
                crate::engine::turn::switch_pokemon_silent(g, 1 - p, t.s)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
