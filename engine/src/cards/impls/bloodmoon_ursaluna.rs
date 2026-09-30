//! Bloodmoon Ursaluna (SFA 25): Battle-Hardened — when you play this Pokémon
//! from your hand onto your Bench, you may attach up to 2 Basic [F] Energy
//! from your hand to it. Mad Bite — 100+, 30 more per damage counter on the
//! opponent's Active.
//!
//! Twinleaf: a ConfirmPrompt first; answering yes without a Basic [F] Energy
//! in hand throws CANNOT_USE_POWER in the callback.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BloodmoonUrsaluna", mask: mask(&[k::PLAY_POKEMON, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn fighting_filter() -> Filter {
    let mut filter = Filter::super_type(SuperType::Energy);
    filter.energy_type = Some(EnergyType::Basic as u8);
    filter.name = Some("Fighting Energy");
    filter
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            let p = p as usize;
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
        }
    }
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { opp, .. } = *g.e(e) {
            let o = opp as usize;
            let a = g.st.players[o].active;
            let d = g.st.slot(o, a).damage;
            if d > 0 {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage = 100 + d * 3;
                }
            }
        }
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
            let has = g.st.players[p].hand.iter().any(|c| {
                let d = g.st.cdef(c);
                d.is_energy() && d.energy_type == EnergyType::Basic as u8 && d.provides.contains(&ct::FIGHTING)
            });
            if !has {
                bail!("CANNOT_USE_POWER");
            }
            let (sp, s) = match g.st.locate(me) {
                Some(ListRef::Slot(sp, s)) => (sp, s),
                _ => return Ok(()),
            };
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.l[0] = sp;
            nf.l[1] = s;
            choose_cards(g, p, "CHOOSE_CARD_TO_ATTACH", ListRef::Hand(p as u8), fighting_filter(), ChooseCardsOpts::new(0, 2, false), Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            if !cards.is_empty() {
                move_cards(g, ListRef::Hand(p as u8), ListRef::Slot(f.l[0], f.l[1]), &cards, me)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
