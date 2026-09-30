//! N's Zoroark ex (JTG): Trade - discard a card from your hand to draw 2,
//! once during your turn; Night Joker - choose 1 of your Benched N's
//! Pokémon's attacks and use it as this attack.
//!
//! Night Joker runs Twinleaf's COPY_ATTACK_FROM_POKEMON_LIST (see
//! `copy_attack.rs`): the copied attack is a separate AttackEffect for a
//! clone of the source's attack, with the source card's handler delegated to
//! this card, all before Night Joker's own attack animation and damage step.
use crate::cards::prelude::*;
use crate::copy_attack::copy_attack_from_pokemon_list;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "NsZoroarkex",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::POWER, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn trade() -> crate::markers::MarkerName {
    marker!("TRADE_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(trade(), me);
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].marker.remove_from(trade(), me);
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].hand.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        if g.st.players[p].marker.has_from(trade(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Hand(p as u8), Filter::none(), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut ns: Vec<CardId> = Vec::new();
        for &b in g.st.players[p].bench.iter() {
            if let Some(c) = g.st.slot_pokemon(p, b) {
                let d = g.st.cdef(c);
                if d.has_tag(tag::NS) && d.name != "N's Zoroark ex" {
                    ns.push(c);
                }
            }
        }
        if ns.is_empty() {
            return Ok(());
        }
        return copy_attack_from_pokemon_list(g, e, &ns, false);
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let cards: Vec<CardId> = results.first().copied().unwrap_or(Res::Null).cards().to_vec();
    ability_used(g, p, me);
    g.st.players[p].marker.add(trade(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &cards, me)?;
    g.run_fx(Effect::MoveCards {
        source: ListRef::Deck(p as u8),
        destination: ListRef::Hand(p as u8),
        cards: None,
        count: Some(2),
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    Ok(())
}
