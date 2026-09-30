//! Hydreigon ex (SSP, Tera): Crashing Headbutt — 200; discard the top 3
//! cards of your opponent's deck. Obsidian — 130; also 130 damage to 2 of
//! your opponent's Benched Pokémon.
//!
//! Twinleaf: THIS_ATTACK_DOES_X_DAMAGE_TO_X_OF_YOUR_OPPONENTS_POKEMON with
//! min = max = min(2, benched). `AttackEffect.target` is never set, so the
//! `effect.target === effect.opponent.active` branch never fires and every
//! target gets a PutDamageEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Hydreigonex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    tera_rule(g, e, me);
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        move_count_from(g, ListRef::Deck(o as u8), ListRef::Discard(o as u8), 3, me)?;
    }
    if was_attack_used(g, e, 1, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let pl = &g.st.players[o];
        let benched = pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards.is_empty()).count();
        if benched == 0 {
            return Ok(());
        }
        let n = benched.min(2) as u8;
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: n, max: n, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let mut r = Ok(());
    for t in targets {
        r = put_damage(g, atk, 130, t);
        if r.is_err() {
            break;
        }
    }
    g.release_fx(atk);
    r
}
