//! Mamoswine ex (JTG): Mammoth Hauler — once during your turn, you may
//! search your deck for a Pokémon, reveal it, and put it into your hand,
//! then shuffle. Rumbling March — 180 + 40 for each Stage 2 Pokémon on your
//! Bench.
//!
//! Twinleaf: MAMMOTH_RIDE_MARKER (player marker, sourced by this card) is
//! removed on this card's PlayPokemonEffect and at every EndTurnEffect; the
//! power throws POWER_ALREADY_USED while it is set. ChooseCardsPrompt
//! (Pokémon, min 0, max 1, no cancel, even on an empty deck) → MOVE_CARDS
//! deck→hand, ShowCardsPrompt to the opponent (if any card), ABILITY_USED
//! board effect, then a ShuffleDeckPrompt whose callback applies the order
//! and adds the marker.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "Mamoswineex",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::POWER, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn mammoth() -> crate::markers::MarkerName {
    marker!("MAMMOTH_RIDE_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(mammoth(), me);
        }
    }
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].marker.remove_from(mammoth(), me);
        return Ok(());
    }
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(mammoth(), me) {
            bail!("POWER_ALREADY_USED");
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::super_type(SuperType::Pokemon), ChooseCardsOpts::new(0, 1, false), Cont::Card { card: me, frame: f });
        return Ok(());
    }
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let pl = &g.st.players[p];
            let n = pl
                .bench
                .iter()
                .filter(|b| matches!(g.st.slot_pokemon(p, **b), Some(c) if g.st.cdef(c).stage == Stage::Stage2 as u8))
                .count() as i32;
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = 180 + n * 40;
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
            let cards: Vec<CardId> = first.cards().to_vec();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            if !cards.is_empty() {
                let oid = g.player_id(1 - p);
                g.prompt(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
            }
            ability_used(g, p, me);
            let mut nf = f;
            nf.stage = 2;
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            g.st.players[p].marker.add(mammoth(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            Ok(())
        }
        _ => Ok(()),
    }
}
