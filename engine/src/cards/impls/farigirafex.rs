//! Farigiraf ex (TEF, Tera): Armor Tail — prevent all damage done to this
//! Pokémon by attacks from your opponent's Basic Pokémon ex. Dirty Beam —
//! 160; also 30 damage to 1 of your opponent's Benched Pokémon.
//!
//! Twinleaf quirks kept: Armor Tail reacts to every PutDamageEffect whose
//! target list contains this card (any source, any phase) and probes the
//! lock with a real PowerEffect for the *attacking* player; when that probe
//! throws, reduceEffect returns early, skipping the Tera bench protection
//! for that PutDamageEffect as well.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Farigirafex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PutDamage { b, .. } = *g.e(e) {
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).cards.contains(me) {
            // `new PowerEffect(player, this.powers[0], this)`: when a copy session runs
            // this code for a copycat without Abilities (Team Rocket's Mimikyu copying
            // Dirty Beam), `this.powers[0]` is undefined, the probe throws and is caught.
            if g.st.cdef(me).powers.is_empty() {
                return Ok(());
            }
            let power = PowerRef { card: me, index: 0 };
            if g.run_fx(Effect::Power { p: b.player, power, card: me, target: None, probe: false }).is_err() {
                return Ok(());
            }
            if let Some(src) = g.st.slot_pokemon(b.source.p as usize, b.source.s) {
                let d = g.st.cdef(src);
                if d.has_tag(tag::POKEMON_EX_LOWER) && d.stage == Stage::Basic as u8 {
                    g.set_prevent(e, true);
                }
            }
        }
    }
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let pl = &g.st.players[o];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    tera_rule(g, e, me);
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let t = results.first().and_then(|r| r.slots().first().copied());
    let r = match t {
        Some(t) => put_damage(g, atk, 30, t),
        None => Ok(()),
    };
    g.release_fx(atk);
    r
}
