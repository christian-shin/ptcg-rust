//! Iron Leaves (TWM): Recovery Net — choose up to 2 Pokémon from your discard
//! pile, reveal them, and put them into your hand. Avenging Edge — 100+; 60
//! more if any of your Pokémon were Knocked Out by attack damage during your
//! opponent's last turn.
//!
//! Twinleaf: Recovery Net does nothing with no Pokémon in the discard pile;
//! otherwise a non-cancellable ChooseCardsPrompt (min 0 since phase 4b R7E:
//! "up to 2" in an attack may take 0, rulings 1721/1778; max min(2, count))
//! and MOVE_CARDS to hand, with no reveal prompt.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "IronLeaves@TWM", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let n = g.st.players[p].discard.iter().filter(|c| g.st.cdef(*c).is_pokemon()).count();
        if n == 0 {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        choose_cards(
            g,
            p,
            "CHOOSE_CARD_TO_HAND",
            ListRef::Discard(p as u8),
            Filter::super_type(SuperType::Pokemon),
            ChooseCardsOpts::new(0, n.min(2) as u8, false),
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].pokemon_knocked_out_by_attack_during_opponents_last_turn {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 60;
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
    let selected: Vec<CardId> = results.first().map(|r| r.cards().to_vec()).unwrap_or_default();
    move_cards(g, ListRef::Discard(p as u8), ListRef::Hand(p as u8), &selected, me)
}
