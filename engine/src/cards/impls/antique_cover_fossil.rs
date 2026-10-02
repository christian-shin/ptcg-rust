//! Antique Cover Fossil (SCR, support card): play this card as a 60 HP [C]
//! Basic Pokémon (at any time during your turn you may discard it from play);
//! Protective Cover — prevent all effects of attacks used by your opponent's
//! Pokémon done to this Pokémon. (Damage is not an effect.)
//!
//! Twinleaf (copied from the Poké Doll pattern, ported as is): the Trainer
//! Ability throws CANNOT_USE_POWER unless this card is the first card of the
//! player's Active slot, then puts it on the BOTTOM of the deck (not the
//! discard pile) and the other attached cards into the discard pile. On its
//! own PlayItemEffect the card reduces a PlayPokemonEffect into the first
//! empty Bench slot; a RetreatEffect with it Active throws. Every attack
//! effect (AbstractAttackEffect) aimed at a slot holding this card as its top
//! Pokémon, with an attacking Pokémon present, is prevented after a lock
//! probe for the owner, except Weakness/Resistance, Put Damage and Deal
//! Damage. No Special Condition handling (unlike Antique Root Fossil).
use super::shuppet::HIDE_N_SNEAK_KINDS;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "AntiqueCoverFossil",
    mask: mask(&HIDE_N_SNEAK_KINDS).or(mask(&[k::POWER, k::PLAY_ITEM, k::RETREAT])),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let list = match g.st.locate(me) {
            Some(l) => l,
            None => bail!("INVALID_GAME_STATE"),
        };
        let a = g.st.players[p].active;
        if g.st.slot(p, a).cards.as_slice().first().copied() != Some(me) {
            bail!("CANNOT_USE_POWER");
        }
        g.run_fx(Effect::MoveCards {
            source: list,
            destination: ListRef::Deck(p as u8),
            cards: Some(List::from_slice(&[me])),
            count: None,
            to_top: false,
            to_bottom: true,
            skip_cleanup: false,
            source_card: NO_CARD,
        })?;
        let rest: Vec<CardId> = match list {
            ListRef::Slot(q, s) => g.st.slot(q as usize, s).cards.iter().filter(|c| *c != me).collect(),
            _ => g.lst(list).iter().copied().filter(|c| *c != me).collect(),
        };
        move_cards(g, list, ListRef::Discard(p as u8), &rest, NO_CARD)?;
    }

    if let Effect::PlayItem { p, card, .. } = *g.e(e) {
        if card == me {
            let pu = p as usize;
            let slots = empty_bench_slots(g, pu);
            let s = match slots.as_slice().first() {
                Some(s) => *s,
                None => bail!("CANNOT_PLAY_THIS_CARD"),
            };
            g.run_fx(Effect::PlayPokemon { p, card: me, target: SlotRef::new(pu, s), slot: SlotType::Board, index: 0 })?;
        }
    }

    if let Effect::Retreat { p, .. } = *g.e(e) {
        if g.st.active_pokemon(p as usize) == Some(me) {
            bail!("CANNOT_RETREAT");
        }
    }

    // Prevent effects of attacks.
    if let Some(b) = g.e(e).atk_base().copied() {
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).cards.contains(me) {
            if g.st.slot_pokemon(t.p as usize, t.s) != Some(me) {
                return Ok(());
            }
            if g.st.slot_pokemon(b.source.p as usize, b.source.s).is_some() {
                if is_ability_blocked(g, t.p as usize, me, None) {
                    return Ok(());
                }
                if matches!(*g.e(e), Effect::ApplyWeakness { .. } | Effect::PutDamage { .. } | Effect::DealDamage { .. }) {
                    return Ok(());
                }
                g.set_prevent(e, true);
            }
        }
    }
    Ok(())
}
