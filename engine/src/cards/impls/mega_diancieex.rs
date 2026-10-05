//! Mega Diancie ex (PFL / ASC): Diamond Coat - this Pokémon takes 30 less
//! damage from attacks (after Weakness and Resistance). Garland Ray - discard
//! up to 2 Energy cards from this Pokémon; 120 damage for each card
//! discarded.
//!
//! Twinleaf: Garland Ray opens a DiscardEnergyPrompt on the Active (any
//! Energy, min 1, max 2, no cancel); a null answer sets the damage to 0,
//! otherwise `damage = 120 * transfers` and each transfer is a MOVE_CARDS to
//! the discard. Diamond Coat: on a PutDamageEffect whose target holds this
//! card as its Pokémon card, during the ATTACK phase and unless blocked for
//! the target's owner, `damage = max(0, damage - 30)`.
//! R7A (ruling 1874): the Energy is chosen first, the damage is done, then the Energy is discarded (`move_cards_after_damage`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaDiancieex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        let o = MoveOpts { allow_cancel: false, min: 1, max: Some(2), ..Default::default() };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_ENERGIES_TO_DISCARD",
            PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter: Filter::super_type(SuperType::Energy), o },
            Cont::Card { card: me, frame: f },
        );
    }
    if let Effect::PutDamage { b, damage, .. } = *g.e(e) {
        if ignores_defender_effects(g, &b) {
            return Ok(());
        }
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).cards.contains(me) {
            if g.st.slot_pokemon(t.p as usize, t.s) != Some(me) {
                return Ok(());
            }
            if g.st.phase != GamePhase::Attack {
                return Ok(());
            }
            let owner = t.p as usize;
            if is_ability_blocked(g, owner, me, None) {
                return Ok(());
            }
            if let Effect::PutDamage { damage: d, .. } = g.e_mut(e) {
                *d = (damage - 30).max(0);
            }
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let e = f.e[0];
    let r = (|| -> R {
        let transfers = match results.first().copied() {
            Some(Res::CardsFrom(t)) => t,
            _ => {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage = 0;
                }
                return Ok(());
            }
        };
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 120 * transfers.len() as i32;
        }
        for (from, c) in transfers.iter().copied() {
            let source = get_target(&g.st, p, from)?;
            move_cards_after_damage(g, e, source.list(), ListRef::Discard(p as u8), &[c], me)?;
        }
        Ok(())
    })();
    g.release_fx(e);
    r
}
