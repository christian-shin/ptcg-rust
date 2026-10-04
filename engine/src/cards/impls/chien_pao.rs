//! Chien-Pao (SSP): Snow Sink - when you play this Pokémon from your hand
//! onto your Bench, you may discard a Stadium in play. Icicle Loop - 120;
//! put an Energy attached to this Pokémon into your hand.
//!
//! Twinleaf quirks kept: the ability check runs while the card is still in
//! hand and the prompt is offered on any turn. Fixed (phase 4b, X1-2): Icicle
//! Loop used to run in the attack handler, i.e. before the damage step, and
//! asked for energy covering [C][C] (up to 2 Energy went to the hand, and a
//! lone 2-unit Special Energy could not be chosen); it now runs on the
//! AfterAttackEffect and asks for exactly 1 attached Energy card.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "ChienPao",
    mask: mask(&[k::PLAY_POKEMON, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card != me {
            return Ok(());
        }
        let p = p as usize;
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        if let Some(stadium) = g.st.stadium_card() {
            let mut f = CardFrame::at(1);
            f.a[0] = stadium as i32;
            let id = g.player_id(p);
            g.prompt(id, "WANT_TO_USE_ABILITY", PromptKind::Confirm, Cont::Card { card: me, frame: f });
        }
        return Ok(());
    }

    if after_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::AfterAttack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        if !g.st.slot(p, a).energies.iter().any(|c| g.st.cdef(c).is_energy()) {
            return Ok(());
        }
        let mut f = CardFrame::at(2);
        f.a[0] = p as i32;
        choose_cards(g, p, "CHOOSE_CARD_TO_HAND", ListRef::Slot(p as u8, a), Filter::super_type(SuperType::Energy), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if !first.as_bool() {
                return Ok(());
            }
            let stadium = f.a[0] as CardId;
            let src = match g.st.locate(stadium) {
                Some(l) => l,
                None => bail!("TypeError: Cannot read properties of undefined"),
            };
            let owner = match src.owner() {
                Some(o) => o,
                None => bail!("TypeError: Cannot read properties of undefined"),
            };
            g.run_fx(Effect::MoveCards {
                source: src,
                destination: ListRef::Discard(owner as u8),
                cards: None,
                count: None,
                to_top: false,
                to_bottom: false,
                skip_cleanup: false,
                source_card: me,
            })?;
            Ok(())
        }
        2 => {
            let p = f.a[0] as usize;
            let cards: Vec<CardId> = first.cards().to_vec();
            let a = g.st.players[p].active;
            move_cards(g, ListRef::Slot(p as u8, a), ListRef::Hand(p as u8), &cards, me)?;
            Ok(())
        }
        _ => Ok(()),
    }
}
