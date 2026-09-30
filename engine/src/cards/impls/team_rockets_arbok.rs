//! Team Rocket's Arbok (DRI): Intimidating Glare — while this Pokémon is
//! your Active, your opponent can't play Pokémon with Abilities from their
//! hand (except Team Rocket's Pokémon). Spinning Tail — 30 damage to each of
//! your opponent's Pokémon.
//!
//! Twinleaf: any PlayPokemonEffect (bench or evolve) by the player whose
//! opponent has this card as the Active top card throws when the played card
//! has an Ability after CheckPokemonPowersEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsArbok", mask: mask(&[k::PLAY_POKEMON, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        let p = p as usize;
        let o = 1 - p;
        let a = g.st.players[o].active;
        if g.st.slot_pokemon(o, a) != Some(me) {
            return Ok(());
        }
        if is_ability_blocked(g, o, me, None) {
            return Ok(());
        }
        let mut powers = SVec::new();
        for i in 0..g.st.cdef(card).powers.len() {
            powers.push(PowerRef { card, index: i as u8 });
        }
        let (pe, _) = g.run_fx(Effect::CheckPokemonPowers { p: p as u8, target: card, powers })?;
        let has_ability = match pe {
            Effect::CheckPokemonPowers { powers, .. } => powers.iter().any(|r| power_type_of(g, *r) == PowerType::Ability as u8),
            _ => false,
        };
        if has_ability && !g.st.cdef(card).has_tag(tag::TEAM_ROCKET) {
            bail!("BLOCKED_BY_ABILITY");
        }
    }
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        for (s, _, _) in for_each_pokemon(g, opp, PlayerType::TopPlayer).iter().copied() {
            damage_opponent_pokemon(g, e, 30, &[SlotRef::new(opp, s)])?;
        }
    }
    Ok(())
}
