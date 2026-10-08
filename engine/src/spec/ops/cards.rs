//! Moving cards between zones, Energy, Prizes (vocabulary v1 "Operations",
//! cards and zones).

use super::super::run::{Flow, Frame};
use super::super::*;
use crate::effects::{AtkBase, Effect, SlotRef};
use crate::game::{Game, R};
use crate::list::*;
use crate::prefabs::*;
use crate::prompts::*;

pub struct MoveSpec {}
pub struct PickSpec {
    pub chooser: Who,
    pub from: ZoneRef,
    pub predicate: Pred,
    pub bounds: Bounds,
}
pub struct DrawSpec {
    pub who: Who,
    pub amount: DrawAmount,
}
pub enum DrawAmount {
    Count(Num),
    /// Draw until the hand has this many cards (no draw if it has as many).
    UntilHandSize(Num),
}
pub struct ShuffleSpec {
    pub zone: ZoneRef,
}
pub struct RevealSpec {}
/// Search a zone for cards matching `pick` and put them somewhere. An attack
/// whose search can't be carried out still resolves (rulings 336, 337, 1790);
/// a Trainer or Ability with nothing to search for can't be used.
pub struct SearchSpec {
    pub pick: PickSpec,
    pub destination: SearchDestination,
    pub msg: &'static str,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SearchDestination {
    /// Onto the chooser's Bench (as played from the zone), at most the open
    /// Bench spaces.
    Bench,
}
pub struct Bounds {
    pub min: Num,
    pub max: Num,
}
pub struct SnapshotSpec {}
pub struct OrderSpec {}
pub struct AttachSpec {}
pub struct MoveEnergySpec {}
pub struct DiscardEnergySpec {
    pub target: SlotExpr,
    pub selection: EnergySelection,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnergySelection {
    /// Every card providing Energy to the Pokémon.
    AllProvided,
}
pub struct PlayFromZoneSpec {}
pub struct PickPrizeSpec {}
pub struct PrizeVisibilitySpec {}
pub struct TakePrizeSpec {}
/// Shuffle a hand into its deck, then draw (the resolving card is not part
/// of the hand).
pub struct HandShuffleDrawSpec {
    pub who: Who,
    pub draw: Num,
}

pub(crate) fn exec(g: &mut Game, me: CardId, f: &mut Frame, op: &Op) -> R<Flow> {
    match op {
        Op::Draw(d) => {
            let p = f.who(d.who);
            let n = match &d.amount {
                DrawAmount::Count(n) => num(g, me, f, n),
                DrawAmount::UntilHandSize(n) => num(g, me, f, n) - g.st.players[p].hand.len() as i32,
            };
            if n > 0 {
                draw_cards(g, p, n as usize)?;
            }
            Ok(Flow::Next)
        }
        Op::Search(s) => search(g, me, f, s),
        Op::Shuffle(s) => {
            let p = f.who(s.zone.0);
            let id = g.player_id(p);
            g.prompt(id, "", PromptKind::ShuffleDeck, f.cont(me, 1));
            Ok(Flow::Suspend)
        }
        Op::DiscardEnergy(d) => {
            let Some(slot) = slot_of(g, me, f, d.target) else { return Ok(Flow::Next) };
            let Some((p, opp, attack, source)) = attack_data(g, f.eff) else { return Ok(Flow::Next) };
            match d.selection {
                EnergySelection::AllProvided => {
                    let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: slot.p, source: slot, energy_map: SVec::new() })?;
                    let mut cards: SVec<CardId, 64> = SVec::new();
                    if let Effect::CheckProvidedEnergy { energy_map, .. } = pe {
                        for m in energy_map.iter() {
                            cards.push(m.card);
                        }
                    }
                    let b = AtkBase { attack_effect: f.eff, player: p, opponent: opp, attack, source, target: slot };
                    g.run_fx(Effect::DiscardCards { b, cards })?;
                }
            }
            Ok(Flow::Next)
        }
        Op::HandShuffleDraw(h) => {
            let p = f.who(h.who);
            let n = num(g, me, f, &h.draw).max(0) as u8;
            shuffle_hand_into_deck_then_draw_ex(g, p, me, NO_CARD, n, Some((me, f.frame_at(1))))?;
            Ok(Flow::Suspend)
        }
        _ => unimplemented!("spec op not implemented yet (ops/cards.rs)"),
    }
}

pub(crate) fn resume(g: &mut Game, _me: CardId, f: &mut Frame, op: &Op, results: &[Res]) -> R<Flow> {
    let first = results.first().copied().unwrap_or(Res::Null);
    match op {
        Op::Search(s) => {
            let p = f.who(s.pick.chooser);
            let mut chosen: SVec<CardId, 64> = SVec::new();
            for c in first.cards() {
                chosen.push(*c);
            }
            match s.destination {
                SearchDestination::Bench => {
                    let open = empty_bench_slots(g, p);
                    for (c, slot) in chosen.iter().zip(open.iter()) {
                        g.run_fx(Effect::PlayPokemonFromDeck { p: p as u8, card: *c, target: SlotRef::new(p, *slot) })?;
                    }
                }
            }
            Ok(Flow::Next)
        }
        Op::Shuffle(s) => {
            if let Res::Order(o) = first {
                let p = f.who(s.zone.0);
                crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
            }
            Ok(Flow::Next)
        }
        // Resumed after the prefab's draw.
        Op::HandShuffleDraw(_) => Ok(Flow::Next),
        _ => Ok(Flow::Next),
    }
}

pub(crate) fn choice(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op) -> R<Flow> {
    Ok(Flow::Next)
}

pub(crate) fn resume_choice(_g: &mut Game, _me: CardId, _f: &mut Frame, _op: &Op, _results: &[Res]) -> R<Flow> {
    Ok(Flow::Next)
}

pub(crate) fn implied_ok(g: &Game, _me: CardId, f: &Frame, op: &Op) -> bool {
    match op {
        Op::Search(x) => {
            let p = f.who(x.pick.chooser);
            let from = zone_ref(f, x.pick.from);
            let room = match x.destination {
                SearchDestination::Bench => !empty_bench_slots(g, p).is_empty(),
            };
            !g.lst(from).is_empty() && room
        }
        _ => true,
    }
}

fn search(g: &mut Game, me: CardId, f: &mut Frame, s: &SearchSpec) -> R<Flow> {
    let p = f.who(s.pick.chooser);
    let from = zone_ref(f, s.pick.from);
    let open = match s.destination {
        SearchDestination::Bench => empty_bench_slots(g, p).len() as i32,
    };
    if g.lst(from).is_empty() || open == 0 {
        return Ok(Flow::Next);
    }
    let max = num(g, me, f, &s.pick.bounds.max).min(open).max(0) as u8;
    let min = num(g, me, f, &s.pick.bounds.min).min(max as i32).max(0) as u8;
    let mut opts = ChooseCardsOpts::new(min, max, false);
    for (i, c) in g.lst(from).iter().enumerate() {
        if !pred(g, *c, &s.pick.predicate) {
            opts.blocked.push(i as u8);
        }
    }
    choose_cards(g, p, s.msg, from, Filter::none(), opts, f.cont(me, 1));
    Ok(Flow::Suspend)
}
