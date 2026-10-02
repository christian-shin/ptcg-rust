//! Iron Crown ex (TEF): Cobalt Command — your Future Pokémon's attacks,
//! except any Iron Crown ex, do 20 more damage to your opponent's Active
//! Pokémon (before applying Weakness and Resistance). Twin Shotels — 50
//! damage to 2 of your opponent's Pokémon; the damage isn't affected by
//! Weakness or Resistance, or by any effects on those Pokémon.
//!
//! Twinleaf: Cobalt Command reacts to every DealDamageEffect of a player that
//! has this card in play (copies stack): attack phase, a Future source that
//! isn't named Iron Crown ex, the Defending Active as target, positive
//! damage, ability not blocked for that player. Twin Shotels opens a
//! non-cancellable ChoosePokemonPrompt (1-2 targets, Active or Bench); for
//! each chosen Pokémon it sets `ignoreWeakness/ignoreResistance` on the
//! attack, reduces its own ApplyWeaknessEffect(50) (own flags unset, target =
//! the opponent's Active even for a Benched pick), zeroes the attack damage,
//! adds the result to the chosen Pokémon and reduces an AfterDamageEffect
//! (target = the opponent's Active).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "IronCrownex", mask: mask(&[k::ATTACK, k::DEAL_DAMAGE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 2, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }

    if let Effect::DealDamage { b, damage } = *g.e(e) {
        let player = b.player as usize;
        let in_play = for_each_pokemon(g, player, PlayerType::BottomPlayer).iter().any(|x| x.1 == me);
        if in_play && g.st.phase == GamePhase::Attack {
            let opponent = 1 - player;
            let source = match g.st.slot_pokemon(b.source.p as usize, b.source.s) {
                Some(c) => c,
                None => bail!("TypeError: Cannot read properties of undefined"),
            };
            let d = g.st.cdef(source);
            if d.has_tag(tag::FUTURE)
                && d.name != "Iron Crown ex"
                && b.target.p as usize == opponent
                && b.target.s == g.st.players[opponent].active
                && damage > 0
                && !is_ability_blocked(g, player, me, None)
            {
                if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                    *damage += 20;
                }
            }
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let r = (|| -> R {
        let (p, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let o = opp as usize;
        for t in targets {
            if let Effect::Attack { ignore_weakness, ignore_resistance, .. } = g.e_mut(atk) {
                *ignore_weakness = true;
                *ignore_resistance = true;
            }
            let oa = SlotRef::new(o, g.st.players[o].active);
            let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: oa };
            let (w, _) = g.run_fx(Effect::ApplyWeakness { b, damage: 50, ignore_weakness: false, ignore_resistance: false })?;
            let damage = match w {
                Effect::ApplyWeakness { damage, .. } => damage,
                _ => 50,
            };
            if let Effect::Attack { damage, .. } = g.e_mut(atk) {
                *damage = 0;
            }
            if damage > 0 {
                g.st.players[t.p as usize].slots[t.s as usize].damage += damage;
                g.run_fx(Effect::AfterDamage { b, damage })?;
            }
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
