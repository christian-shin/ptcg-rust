//! Mr. Mime (TEF): Look-Alike Show — your opponent reveals their hand; you may
//! use the effect of a Supporter card you find there as the effect of this
//! attack. Eerie Wave — 20; your opponent's Active Pokémon is now Confused.
//!
//! The one choice no vocabulary item expresses (a card chosen from the opponent's hand whose effect
//! is then run as this attack's, re-asking when that effect can't be used): the card's own steps,
//! a Custom op. The Supporter's effect is not "playing" it: the one-Supporter rule is bypassed
//! (rulings 1727, 1844, 1853); a Supporter whose effect can't be used right now is blocked in the
//! prompt, which is asked again.
use crate::cards::prelude::*;
use crate::spec::prelude::*;
use crate::spec::run::{Flow, Frame, NONE};

pub static SPEC: CardSpec = CardSpec {
    class: "MrMime",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Custom(CustomSpec { exec: choose_supporter, resume: copy_effect }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(inflict(&[SpecialCondition::Confused], Cause::Attack))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

/// The choice among the Supporters in the opponent's hand (none is also an answer).
fn choose_supporter(g: &mut Game, me: CardId, f: &mut Frame) -> R<Flow> {
    let p = f.p as usize;
    let opp = 1 - p;
    let mut filter = Filter::super_type(SuperType::Trainer);
    filter.trainer_type = Some(TrainerType::Supporter as u8);
    let mut opts = ChooseCardsOpts::new(0, 1, false);
    if f.cards[0] != NONE {
        let tried: Vec<CardId> = g.lst(ListRef::Temp(f.cards[0])).to_vec();
        for (i, c) in g.st.players[opp].hand.iter().enumerate() {
            if tried.contains(&c) {
                opts.blocked.push(i as u8);
            }
        }
    }
    choose_cards(g, p, "CHOOSE_CARD_TO_COPY_EFFECT", ListRef::Hand(opp as u8), filter, opts, f.cont(me, 1));
    Ok(Flow::Suspend)
}

fn copy_effect(g: &mut Game, me: CardId, f: &mut Frame, results: &[Res]) -> R<Flow> {
    let p = f.p as usize;
    let Some(card) = results.first().and_then(|r| r.cards().first().copied()) else { return Ok(Flow::Next) };
    let supporter_turn = g.st.players[p].supporter_turn;
    g.st.players[p].supporter_turn = 0;
    let r = g.run_fx(Effect::Trainer { p: p as u8, card, target: None, via_attack: true });
    g.st.players[p].supporter_turn = supporter_turn;
    match r {
        Ok(_) => Ok(Flow::Next),
        // A GameError (not a TypeError) means this Supporter's effect can't be used now: choose another.
        Err(e) if !e.0.starts_with("TypeError") => {
            if f.cards[0] == NONE {
                if let ListRef::Temp(i) = g.alloc_temp(&[]) {
                    f.cards[0] = i;
                }
            }
            let list = ListRef::Temp(f.cards[0]);
            let mut tried: Vec<CardId> = g.lst(list).to_vec();
            tried.push(card);
            g.lst_mut(list).set_from(&tried);
            choose_supporter(g, me, f)
        }
        Err(e) => Err(e),
    }
}
