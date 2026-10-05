//! Scream Tail ex (TWM): Scream — can only be used if you go second, during
//! your first turn; during your opponent's next turn, they can't play any
//! Supporter cards from their hand. Crunch — 120; discard an Energy from your
//! opponent's Active Pokémon.
//!
//! Twinleaf: Scream throws CANNOT_USE_ATTACK unless `state.turn === 2`. Crunch
//! opens a non-cancellable ChooseCardsPrompt on the Defending Pokémon (when it
//! has an Energy card) and reduces a DiscardCardsEffect in the callback.
//!
//! Fixed (phase 4b, R2): Crunch discarded in the attack handler, before the
//! damage (the Defending Pokémon's Spiky Energy was already gone); it now runs
//! in AfterAttackEffect with a fresh AttackEffect's data.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ScreamTailex", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if g.st.turn != 2 {
            bail!("CANNOT_USE_ATTACK");
        }
        return opponent_cannot_play_cards(g, e, crate::effects::play_lock::SUPPORTER);
    }

    if after_attack_used(g, e, 1, me) {
        let (p, o) = match *g.e(e) {
            Effect::AfterAttack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let a = g.st.players[o].active;
        if !g.st.slot(o, a).cards.iter().any(|c| g.st.cdef(c).is_energy()) {
            return Ok(());
        }
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        choose_cards(
            g,
            p,
            "CHOOSE_CARD_TO_DISCARD",
            ListRef::Slot(o as u8, a),
            Filter::super_type(SuperType::Energy),
            ChooseCardsOpts::new(1, 1, false),
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let r = (|| -> R {
        let card = match results.first().and_then(|r| r.cards().first()) {
            Some(c) => *c,
            None => bail!("TypeError: Cannot read properties of undefined"),
        };
        let (p, opp, attack, source) = match attack_data(g, atk) {
            Some(d) => d,
            None => return Ok(()),
        };
        let a = g.st.players[opp as usize].active;
        let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: SlotRef::new(opp as usize, a) };
        let mut cards = SVec::new();
        cards.push(card);
        g.run_fx(Effect::DiscardCards { b, cards })?;
        Ok(())
    })();
    g.release_fx(atk);
    r
}
