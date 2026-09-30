//! Noctowl (SCR): Jewel Seeker — when you play this Pokémon from your hand to
//! evolve 1 of your Pokémon, if you have any Tera Pokémon in play, you may
//! search your deck for up to 2 Trainer cards, reveal them, and put them into
//! your hand. Then, shuffle your deck. Speed Wing — 60.
//!
//! Twinleaf: runs on its PlayPokemonEffect (before the evolution resolves).
//! Empty deck → nothing; the per-instance marker (set once the reveal is
//! acknowledged, cleared at any EndTurn) makes a later play of the same card
//! throw. The reveal ShowCardsPrompt is created even for 0 cards, and the
//! ShuffleDeckPrompt is created right after it (not after its answer): the
//! cards move to the hand when the reveal is answered.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Noctowl",
    mask: mask(&[k::END_TURN, k::PLAY_POKEMON]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn jewel() -> crate::markers::MarkerName {
    crate::marker!("JEWEL_HUNT_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        g.st.players[p as usize].marker.remove_from(jewel(), me);
    }
    let p = match *g.e(e) {
        Effect::PlayPokemon { p, card, .. } if card == me => p as usize,
        _ => return Ok(()),
    };
    if g.st.players[p].deck.is_empty() {
        return Ok(());
    }
    if g.st.players[p].marker.has_from(jewel(), me) {
        bail!("POWER_ALREADY_USED");
    }
    let pl = &g.st.players[p];
    let mut tera = 0;
    if let Some(c) = g.st.slot_pokemon(p, pl.active) {
        if g.st.cdef(c).has_tag(tag::POKEMON_TERA) {
            tera += 1;
        }
    }
    for &b in pl.bench.iter() {
        if let Some(c) = g.st.slot_pokemon(p, b) {
            if g.st.cdef(c).has_tag(tag::POKEMON_TERA) {
                tera += 1;
            }
        }
    }
    if tera == 0 {
        return Ok(());
    }
    if is_ability_blocked(g, p, me, None) {
        return Ok(());
    }
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
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
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let opts = ChooseCardsOpts::new(0, 2, false);
            choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Deck(p as u8), Filter::super_type(SuperType::Trainer), opts, Cont::Card { card: me, frame: nf });
            Ok(())
        }
        2 => {
            let cards: Vec<CardId> = first.cards().to_vec();
            let mut nf = CardFrame::at(3);
            nf.a[0] = p as i32;
            nf.a[1] = cards.first().map(|c| *c as i32).unwrap_or(-1);
            nf.a[2] = cards.get(1).map(|c| *c as i32).unwrap_or(-1);
            let oid = g.player_id(1 - p);
            g.prompt_group(&[(oid, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards)], Cont::Card { card: me, frame: nf });
            let mut sf = CardFrame::at(4);
            sf.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: sf });
            Ok(())
        }
        3 => {
            let cards: Vec<CardId> = [f.a[1], f.a[2]].iter().filter(|c| **c >= 0).map(|c| *c as CardId).collect();
            move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
            g.st.players[p].marker.add(jewel(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            Ok(())
        }
        4 => {
            if let Res::Order(o) = first {
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
