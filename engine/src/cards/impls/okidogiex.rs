//! Okidogi ex (SFA): Poisonous Musculature — search your deck for up to 2
//! Basic [D] Energy and attach them to this Pokémon, shuffle; if you attached
//! any, this Pokémon is now Poisoned. Chain-Crazed — 130+; 130 more if this
//! Pokémon is Poisoned.
//!
//! Twinleaf: the Poison is an AddSpecialConditionsEffect aimed at the
//! attacker's Active; the shuffle is a bare ShuffleDeckPrompt (no wait).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Okidogiex", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        // findCardList(state, this)
        let slot = match g.st.locate(me) {
            Some(ListRef::Slot(sp, s)) => (sp, s),
            _ => return Ok(()),
        };
        let mut filter = Filter::super_type(SuperType::Energy);
        filter.energy_type = Some(EnergyType::Basic as u8);
        filter.name = Some("Darkness Energy");
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.e[0] = e;
        f.l[0] = slot.0;
        f.l[1] = slot.1;
        choose_cards(g, p, "CHOOSE_CARD_TO_ATTACH", ListRef::Deck(p as u8), filter, ChooseCardsOpts::new(0, 2, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            if g.st.slot(p, a).special_conditions.contains(&(SpecialCondition::Poisoned as u8)) {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 130;
                }
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
    let atk = f.e[0];
    let r = (|| -> R {
        let cards: Vec<CardId> = results.first().copied().unwrap_or(Res::Null).cards().to_vec();
        if !cards.is_empty() {
            move_cards(g, ListRef::Deck(p as u8), ListRef::Slot(f.l[0], f.l[1]), &cards, me)?;
            if let Effect::Attack { p: ap, opp, attack, source, .. } = *g.e(atk) {
                let pp = ap as usize;
                let target = SlotRef::new(pp, g.st.players[pp].active);
                let b = AtkBase { attack_effect: atk, player: ap, opponent: opp, attack, source, target };
                let mut conditions = SVec::new();
                conditions.push(SpecialCondition::Poisoned as u8);
                g.run_fx(Effect::AddSpecialConditions { b, conditions, poison_damage: None, burn_damage: None, confusion_damage: None })?;
            }
        }
        let id = g.player_id(p);
        g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
        Ok(())
    })();
    g.release_fx(atk);
    r
}
