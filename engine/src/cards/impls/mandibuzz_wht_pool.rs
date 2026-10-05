//! Mandibuzz (WHT 64): Look for Prey — once during your turn, your opponent
//! reveals their hand, and you put a Basic Pokémon with 70 HP or less that you
//! find there onto your opponent's Bench. Cutting Wind — 90.
//!
//! Twinleaf: the marker is removed on PlayPokemon of this card and at the end
//! of each turn; BLOCKED_BY_EFFECT / POWER_ALREADY_USED / CANNOT_USE_POWER
//! (opponent's hand empty) are thrown before the marker + ABILITY_USED. With no
//! open Bench slot or no eligible card the hand is only shown (ShowCardsPrompt);
//! else a mandatory ChooseCardsPrompt on the opponent's hand (min 1, max 1,
//! every ineligible index blocked); MOVE_CARDS onto the first open slot and
//! `pokemonPlayedTurn = turn`.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "MandibuzzWHTPool",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn mk() -> crate::markers::MarkerName {
    marker!("WHT_MANDIBUZZ_LOOK_FOR_PREY")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(mk(), me);
        }
    }
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let o = 1 - p;
        if is_ability_blocked(g, p, me, None) {
            bail!("BLOCKED_BY_EFFECT");
        }
        if g.st.players[p].marker.has_from(mk(), me) {
            bail!("POWER_ALREADY_USED");
        }
        if g.st.players[o].hand.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        // Phase 4b R7E (rulings 46, 70, 1634): a full opposing Bench is public knowledge, so the Ability
        // can't be used.
        if empty_bench_slots(g, o).is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        use_ability_once_per_turn(g, p, mk(), me)?;
        ability_used(g, p, me);

        let slots = empty_bench_slots(g, o);
        let mut opts = ChooseCardsOpts::new(1, 1, false);
        let mut blocked_n = 0;
        for (i, c) in g.st.players[o].hand.iter().enumerate() {
            let d = g.st.cdef(c);
            if !(d.is_pokemon() && d.stage == Stage::Basic as u8 && d.hp <= 70) {
                opts.blocked.push(i as u8);
                blocked_n += 1;
            }
        }
        let has_target = blocked_n < g.st.players[o].hand.len();
        if slots.is_empty() || !has_target {
            let id = g.player_id(p);
            g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Noop);
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.l[0] = slots.as_slice()[0];
        let filter = Filter { super_type: Some(SuperType::Pokemon as u8), stage: Some(Stage::Basic as u8), ..Filter::none() };
        choose_cards(g, p, "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ListRef::Hand(o as u8), filter, opts, Cont::Card { card: me, frame: f });
        return Ok(());
    }
    remove_marker_at_end_of_turn(g, e, mk(), me);
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    let cards: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    if cards.is_empty() {
        return Ok(());
    }
    let s = f.l[0] as SlotId;
    move_cards(g, ListRef::Hand(o as u8), ListRef::Slot(o as u8, s), &cards[..1], me)?;
    let turn = g.st.turn;
    g.st.players[o].slots[s as usize].pokemon_played_turn = turn;
    Ok(())
}
