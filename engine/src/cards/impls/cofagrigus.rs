//! Cofagrigus (SSP): Law of the Underworld — put 6 damage counters on each
//! Pokémon that has an Ability (both yours and your opponent's). Spooky
//! Shot — 100.
//!
//! Twinleaf: for each of the player's Pokémon (Active then Bench), then the
//! opponent's, a CheckPokemonPowersEffect is run and, if any power is an
//! Ability, a PutCountersEffect of 60 is applied.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Cofagrigus@Cofagrigus SSP", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn has_ability(g: &mut Game, p: usize, c: CardId) -> R<bool> {
    let mut powers = SVec::new();
    for i in 0..g.st.cdef(c).powers.len() {
        powers.push(PowerRef { card: c, index: i as u8 });
    }
    let (e, _) = g.run_fx(Effect::CheckPokemonPowers { p: p as u8, target: c, powers })?;
    Ok(match e {
        Effect::CheckPokemonPowers { powers, .. } => powers.iter().any(|r| g.st.cdef(r.card).powers[r.index as usize].power_type == PowerType::Ability as u8),
        _ => false,
    })
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    for (q, pt) in [(p as usize, PlayerType::BottomPlayer), (opp as usize, PlayerType::TopPlayer)] {
        for (s, c, _) in for_each_pokemon(g, q, pt).iter().copied() {
            if has_ability(g, q, c)? {
                let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(q, s) };
                g.run_fx(Effect::PutCounters { b, damage: 60 })?;
            }
        }
    }
    Ok(())
}
