//! Mega Starmie ex (POR / M3): Jetting Blow — 120; also 50 damage to 1 of
//! your opponent's Benched Pokémon (PutDamageEffect, no Weakness). Nebula
//! Beam — 210; not affected by Weakness, Resistance, or effects.
//!
//! Twinleaf: Jetting Blow prompts (bench, no cancel) only when the opponent
//! has a Benched Pokémon. Nebula Beam is THIS_ATTACKS_DAMAGE_ISNT_AFFECTED_BY_EFFECTS
//! with `ignoreWeaknessAndResistance` (phase 4b; it used to set
//! `effect.ignoreResistance` on the AttackEffect and build its own
//! ApplyWeaknessEffect with the flags unset, so Weakness and Resistance
//! applied): ApplyWeaknessEffect on `effect.damage`, zero the attack damage,
//! add the damage directly to the opponent's Active, then AfterDamage.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaStarmieex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let pl = &g.st.players[o];
        if pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
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
    }
    if was_attack_used(g, e, 1, me) {
        let d = match *g.e(e) {
            Effect::Attack { damage, .. } => damage,
            _ => return Ok(()),
        };
        super::mega_lopunnyex::shred_ex(g, e, d, true)?;
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let sel = results.first().copied().unwrap_or(Res::Null);
    let sel = sel.slots();
    let r = if sel.is_empty() { Ok(()) } else { put_damage(g, atk, 50, sel[0]) };
    g.release_fx(atk);
    r
}
