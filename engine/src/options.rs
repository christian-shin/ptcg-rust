//! Turn-level options: structural candidates filtered by trial dispatch on a
//! clone (cheap: the whole game is `Copy`). Mirrors the oracle's
//! `turnCandidates` + `legalTurnOptions` exactly, including order.

use crate::effects::*;
use crate::engine::turn::check_attacks_effect;
use crate::game::{Action, Game};
use crate::list::*;
use crate::prompts::{slot_targets, target_json};
use crate::types::*;
use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct TurnOption {
    pub desc: Value,
    pub action: Action,
}

const BOARD: CardTarget = CardTarget::new(PlayerType::BottomPlayer, SlotType::Board, 0);

/// JavaScript default sort order for strings (UTF-16 code units).
fn js_sort(v: &mut Vec<&'static str>) {
    v.sort_by(|a, b| {
        let x: Vec<u16> = a.encode_utf16().collect();
        let y: Vec<u16> = b.encode_utf16().collect();
        x.cmp(&y)
    });
}

pub fn turn_candidates(g: &Game) -> Vec<TurnOption> {
    let mut out = Vec::new();
    let p = g.st.active_player as usize;
    let own = slot_targets(&g.st, p, PlayerType::BottomPlayer, &[SlotType::Active as u8, SlotType::Bench as u8]);
    let pl = &g.st.players[p];
    let first_empty = pl.bench.iter().position(|b| pl.slots[*b as usize].cards.is_empty());
    for (hi, c) in pl.hand.iter().enumerate() {
        let d = g.st.cdef(c);
        let r = g.card_ref(c);
        let mut play = |t: CardTarget| {
            out.push(TurnOption {
                desc: json!({ "a": "play", "card": r, "target": target_json(t) }),
                action: Action::PlayCard { hand_index: hi as u8, target: t },
            })
        };
        if d.is_energy() {
            own.iter().for_each(|t| play(*t));
        } else if d.is_pokemon() {
            if let Some(i) = first_empty {
                play(CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, i as u8));
            }
            if d.stage != Stage::Basic as u8 {
                own.iter().for_each(|t| play(*t));
            }
        } else if d.is_trainer() {
            if d.trainer_type == TrainerType::Tool as u8 {
                own.iter().for_each(|t| play(*t));
            } else {
                play(BOARD);
            }
        }
    }

    // Attack names: active, bench (useOnBench), CheckPokemonAttacks.
    let mut names: Vec<&'static str> = Vec::new();
    let add = |n: &'static str, names: &mut Vec<&'static str>| {
        if !names.contains(&n) {
            names.push(n);
        }
    };
    if let Some(c) = g.st.active_pokemon(p) {
        for a in g.st.cdef(c).attacks {
            add(a.name, &mut names);
        }
    }
    for &b in pl.bench.iter() {
        if let Some(c) = g.st.slot_pokemon(p, b) {
            for a in g.st.cdef(c).attacks.iter().filter(|a| a.use_on_bench) {
                add(a.name, &mut names);
            }
        }
    }
    {
        let mut sim = *g;
        if let Ok((Effect::CheckPokemonAttacks { attacks, .. }, _)) = sim.run_fx(check_attacks_effect(&sim, p)) {
            for a in attacks.iter() {
                add(g.st.cdef(a.card).attacks[a.index as usize].name, &mut names);
            }
        }
    }
    js_sort(&mut names);
    for n in names {
        out.push(TurnOption { desc: json!({ "a": "attack", "name": n }), action: Action::Attack { name: n } });
    }

    // Abilities on Pokémon in play.
    for t in &own {
        let slot = crate::prompts::get_target(&g.st, p, *t).unwrap();
        if let Some(c) = g.st.slot_pokemon(slot.p as usize, slot.s) {
            let mut pn: Vec<&'static str> = Vec::new();
            for pw in g.st.cdef(c).powers {
                if !pn.contains(&pw.name) {
                    pn.push(pw.name);
                }
            }
            let mut sim = *g;
            let mut powers = SVec::new();
            for i in 0..g.st.cdef(c).powers.len() {
                powers.push(PowerRef { card: c, index: i as u8 });
            }
            if let Ok((Effect::CheckPokemonPowers { powers, .. }, _)) = sim.run_fx(Effect::CheckPokemonPowers { p: p as u8, target: c, powers }) {
                for r in powers.iter() {
                    let n = g.st.cdef(r.card).powers[r.index as usize].name;
                    if !pn.contains(&n) {
                        pn.push(n);
                    }
                }
            }
            js_sort(&mut pn);
            for n in pn {
                out.push(TurnOption {
                    desc: json!({ "a": "ability", "name": n, "source": target_json(*t) }),
                    action: Action::UseAbility { name: n, target: *t },
                });
            }
        }
    }
    // Abilities from hand / discard: no pool card has useFromHand / useFromDiscard.

    if g.st.stadium_card().is_some() {
        out.push(TurnOption { desc: json!({ "a": "stadium" }), action: Action::UseStadium });
    }
    for (i, b) in pl.bench.iter().enumerate() {
        if !pl.slots[*b as usize].cards.is_empty() {
            out.push(TurnOption { desc: json!({ "a": "retreat", "bench": i }), action: Action::Retreat { bench_index: i as u8 } });
        }
    }
    out.push(TurnOption { desc: json!({ "a": "pass" }), action: Action::Pass });
    out
}

/// Legal turn options (deduplicated, in candidate order).
pub fn legal_turn_options(g: &Game) -> Vec<TurnOption> {
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for c in turn_candidates(g) {
        let key = serde_json::to_string(&c.desc).unwrap();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        let mut trial = *g;
        if trial.act(c.action).is_ok() {
            out.push(c);
        }
    }
    out
}

/// Legality for a single action without building descriptors (fast path).
pub fn is_legal(g: &Game, a: Action) -> bool {
    let mut trial = *g;
    trial.act(a).is_ok()
}
