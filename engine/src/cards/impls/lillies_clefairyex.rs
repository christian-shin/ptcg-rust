//! Lillie's Clefairy ex (JTG): Fairy Zone - the Weakness of each of your
//! opponent's [N] Pokémon in play is now [P]. Full Moon Rondo - 20 + 20 for
//! each Benched Pokémon (both players).
//!
//! Twinleaf probes the ability lock for the *target's* owner (not the
//! Clefairy's owner), before checking that this Clefairy is in play.
use crate::cards::prelude::*;
use crate::effects::WeaknessV;

pub static IMPL: CardImpl = CardImpl {
    class: "LilliesClefairyex",
    mask: mask(&[k::CHECK_POKEMON_STATS, k::ATTACK]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckPokemonStats { target, .. } = *g.e(e) {
        let player = target.p as usize;
        let opponent = 1 - player;
        let in_play = g.st.players[opponent].in_play().iter().any(|s| g.st.slot_pokemon(opponent, *s) == Some(me));
        if is_ability_blocked(g, player, me, None) {
            return Ok(());
        }
        let dragon = g.st.slot_pokemon(player, target.s).map(|c| g.st.cdef(c).card_type.contains(&ct::DRAGON)).unwrap_or(false);
        if !in_play || !dragon {
            return Ok(());
        }
        let (fx, _) = g.run_fx(Effect::EffectOfAbility { p: opponent as u8, power: PowerRef { card: me, index: 0 }, card: me, target: Some(target) })?;
        if let Effect::EffectOfAbility { target: Some(_), .. } = fx {
            if let Effect::CheckPokemonStats { weakness, .. } = g.e_mut(e) {
                weakness.clear();
                weakness.push(WeaknessV { card_type: ct::PSYCHIC, value: None });
            }
        }
    }

    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let count = |q: usize| g.st.players[q].bench.iter().filter(|b| !g.st.players[q].slots[**b as usize].cards.is_empty()).count() as i32;
        let total = count(p) + count(o);
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 20 + total * 20;
        }
    }
    Ok(())
}
