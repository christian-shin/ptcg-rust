//! Minccino (SSH): Glance — look at the top card of your opponent's deck.
//! Tail Slap — flip 2 coins, 20 damage for each heads.
//!
//! Twinleaf has two `Minccino` classes; this port is bound to SSH. Glance
//! moves the top card into a scratch list, shows it (non-yielding prompt)
//! and puts it back on top when the prompt resolves.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Minccino@SSH", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, opp) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        if g.st.players[opp].deck.is_empty() {
            return Ok(());
        }
        let top = g.alloc_temp(&[]);
        move_count_from(g, ListRef::Deck(opp as u8), top, 1, me)?;
        let mut f = CardFrame::at(1);
        f.a[0] = opp as i32;
        f.l[0] = match top {
            ListRef::Temp(i) => i,
            _ => unreachable!(),
        };
        let id = g.player_id(p);
        g.prompt(id, "CARDS_SHOWED_BY_EFFECT", PromptKind::ShowCards, Cont::Card { card: me, frame: f });
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(2);
        f.e[0] = e;
        if let Err(err) = coin_flip_sequence(g, p, 2, CoinCb::SequenceCard { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, _results: &[Res]) -> R {
    match f.stage {
        1 => {
            let opp = f.a[0] as u8;
            g.move_to_top_of_destination(ListRef::Temp(f.l[0]), ListRef::Deck(opp));
            Ok(())
        }
        2 => {
            let atk = f.e[0];
            let heads = (f.a[2] as u32).count_ones() as i32;
            if let Effect::Attack { damage, .. } = g.e_mut(atk) {
                *damage = 20 * heads;
            }
            g.release_fx(atk);
            Ok(())
        }
        _ => Ok(()),
    }
}
