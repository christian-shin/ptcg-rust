//! Meowth ex (M3 / POR): Last-Ditch Catch — when played from hand onto the
//! Bench, you may search your deck for a Supporter (Twinleaf: any Trainer,
//! non-Supporters blocked). Tuck Tail — put this Pokémon and all attached
//! cards into your hand.
//!
//! Phase 4b (R6): with the state-level TRUMP_CARD_MARKER set (a "Last-Ditch"
//! Ability was used this turn) the Pokémon is still played, only without the
//! Ability; it used to throw POWER_ALREADY_USED inside the PlayPokemonEffect,
//! so a second Meowth ex could not be benched that turn.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Meowthex",
    mask: mask(&[k::PLAY_POKEMON, k::END_TURN, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn trump() -> crate::markers::MarkerName {
    crate::marker!("TRUMP_CARD_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::PlayPokemon { p, card, .. } if card == me => {
            let p = p as usize;
            if g.st.players[p].deck.is_empty() {
                return Ok(());
            }
            if g.st.players[p].marker.has(trump()) {
                return Ok(());
            }
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            let mut blocked: u64 = 0;
            for (i, c) in g.st.players[p].deck.iter().enumerate() {
                let d = g.st.cdef(c);
                if d.is_trainer() && d.trainer_type != TrainerType::Supporter as u8 {
                    blocked |= 1u64 << i;
                }
            }
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            f.a[1] = blocked as u32 as i32;
            f.a[2] = (blocked >> 32) as u32 as i32;
            confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
        }
        Effect::EndTurn { p } => {
            let m = &mut g.st.players[p as usize].marker;
            if m.has(trump()) {
                m.remove(trump());
            }
        }
        Effect::AfterAttack { p, attack, .. } if attack == my_attack(g, me, 0) => {
            let p = p as usize;
            let a = g.st.players[p].active;
            move_pokemon_off_board(g, SlotRef::new(p, a), ListRef::Hand(p as u8), me)?;
        }
        _ => {}
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    if !results.first().map(|r| r.as_bool()).unwrap_or(false) {
        return Ok(());
    }
    let p = f.a[0] as usize;
    ability_used(g, p, me);
    g.st.players[p].marker.add_to_state(trump());
    let mut opts = ChooseCardsOpts::new(0, 1, false);
    opts.blocked = Blocked((f.a[1] as u32 as u64) | ((f.a[2] as u32 as u64) << 32));
    search_deck_for_cards_to_hand(g, p, me, Filter::super_type(SuperType::Trainer), opts);
    Ok(())
}
