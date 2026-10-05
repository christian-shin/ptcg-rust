//! Fan Rotom (SCR): Fan Call — once during your first turn, search your deck
//! for up to 3 [C] Pokémon with 100 HP or less, reveal them, and put them
//! into your hand, then shuffle (1 Fan Call per turn). Assault Landing —
//! 70; does nothing if there is no Stadium in play.
//!
//! Twinleaf: `player.usedFanCall` is set as soon as the Ability is used (phase
//! 4b: it used to be set only when cards were taken, so the Ability could be
//! used again after taking none) and cleared at every end of turn; "first turn"
//! is `state.turn <= 2`. ABILITY_USED is set in the choose callback; the
//! ShuffleDeckPrompt follows the (non-yielding) ShowCardsPrompt with no wait.
//! Assault Landing sets the damage to exactly 0 or 70.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "FanRotom", mask: mask(&[k::END_TURN, k::POWER, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].used_fan_call = false;
        return Ok(());
    }
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].used_fan_call {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.turn > 2 {
            bail!("CANNOT_USE_POWER");
        }
        g.st.players[p].used_fan_call = true;
        let mut opts = ChooseCardsOpts::new(0, 3, false);
        let mut n = 0u8;
        for (i, c) in g.st.players[p].deck.iter().enumerate() {
            let d = g.st.cdef(c);
            if d.is_pokemon() && d.card_type.contains(&ct::COLORLESS) && d.hp <= 100 {
                n += 1;
            } else {
                opts.blocked.push(i as u8);
            }
        }
        opts.max_pokemons = Some(n.min(3));
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::super_type(SuperType::Pokemon), opts, Cont::Card { card: me, frame: f });
    }
    if was_attack_used(g, e, 0, me) {
        let has_stadium = g.st.stadium_card().is_some();
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = if has_stadium { 70 } else { 0 };
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
            ability_used(g, p, me);
            if !cards.is_empty() {
                let top = g.alloc_temp(&cards);
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                nf.l[0] = match top {
                    ListRef::Temp(i) => i,
                    _ => unreachable!(),
                };
                let oid = g.player_id(1 - p);
                g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
            }
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: p as u8 });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = g.lst(ListRef::Temp(f.l[0])).to_vec();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            Ok(())
        }
        _ => Ok(()),
    }
}
