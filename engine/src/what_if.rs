//! What-if queries for a host's search: what an attack would do now, without playing the turn out.
//!
//! [`attack`] uses an attack on copies of the game through the engine's own routine (`engine::turn::use_attack`, the one
//! the UseAttack turn action takes), with the UseAttack event marked `what_if`: the event's checks are not made (the
//! Energy, the locks on using attacks, the first-turn rule, the Special Conditions: no Confusion flip), and the attack
//! stops where it would check Knock Outs and end the turn (`attack::finish_attack`). Everything else is the attack as
//! played: its text before the damage, the damage calculation (APR, Damage Calculation Order 1-6: the attacker-side
//! modifiers; Weakness and Resistance for a Pokémon in an Active Spot only, APR II B-08; the defender-side modifiers; the
//! preventions of damage), its effects after the damage and its triggers (APR E-03). The query reads the damage there,
//! then lets the Knock Out check run (attack step 7, APR D: the Rule Box Prize cards and every Prize adjustment are the
//! KnockOut routine's) and reads the Prize cards taken. There is no second, parallel damage formula.
//!
//! Chance: a copy's generator is scripted (`Rng::scripted`) and the coin sequences are enumerated (see [`attack`]).
//! Choices: a question asked inside the evaluation is answered with its first valid answer and listed in the outcome.
//! Extra Energy: the copies can have Energy cards attached to the attacker before the attack, as if already attached
//! (see [`attack`]), for the damage an attack would do with more Energy.

use crate::carddb::DefId;
use crate::game::{Game, GameError, Pending, R};
use crate::list::CardId;
use crate::purpose::PromptPurpose;
use crate::place::Zone;
use crate::state::{AttackRef, ListRef};
use crate::types::SuperType;
use std::collections::VecDeque;

/// One outcome of a what-if attack: what happened on one class of coin sequences.
#[derive(Clone, Debug, PartialEq)]
pub struct AttackOutcome {
    /// The probability of the coin sequences that give this outcome (1 without coins).
    pub probability: f64,
    /// The damage the attack did to each Pokémon in play when it was used (the Damage events, after every modifier and
    /// prevention; damage counters placed are not damage, APR C-07), nonzero entries only, in table order (the attacking
    /// player's Pokémon first, Active first).
    pub damage: Vec<(CardId, i32)>,
    /// The damage the attack did to the target.
    pub target_damage: i32,
    /// The target's HP left when the attack was over, before the Knock Out check (0 when it left play).
    pub target_hp_left: i32,
    /// The Pokémon the attack's Knock Out check Knocked Out.
    pub knocked_out: Vec<CardId>,
    /// The target is one of them.
    pub target_knocked_out: bool,
    /// The Prize cards each player took at the Knock Out check (indexed by player).
    pub prizes: [u8; 2],
    /// The game ended: the winner (`Game::st.winner`).
    pub winner: Option<i8>,
    /// The questions answered with their first valid answer, in order: the player asked and why. The outcome assumes
    /// those answers (a choice inside the attack; the Knock Out check's Prize card and new Active Pokémon choices, which
    /// change none of the numbers above).
    pub asked: Vec<(u8, PromptPurpose)>,
}

/// What an attack would do now ([`attack`]).
#[derive(Clone, Debug)]
pub struct AttackWhatIf {
    /// The attack used and whether it is a Benched Pokémon's, used as the Active Pokémon's own.
    pub attack: AttackRef,
    pub copied: bool,
    /// The attacker was on the Bench and was put in the Active Spot for the evaluation.
    pub attacker_moved: bool,
    /// The target is the opponent's Active Pokémon (the Defending Pokémon the attack's damage is done to); a Benched
    /// target takes only what the attack's text does to it.
    pub target_defending: bool,
    /// The outcomes, merged (outcomes equal but for their probability are one), in the order they were found.
    pub outcomes: Vec<AttackOutcome>,
    /// The evaluations run (one per coin sequence class).
    pub runs: usize,
    /// The probability of the coin sequences not evaluated (`max_runs` reached, or more than 32 coins flipped).
    pub unexplored: f64,
    /// The extra Energy cards attached to the attacker on the copies, in the order asked: each card's definition and the
    /// zone of its owner it was taken from.
    pub extra_attached: Vec<(DefId, Zone)>,
}

/// The spot of `c` if it is the Pokémon of a spot in play (its top card).
fn spot_of(g: &Game, c: CardId) -> Option<crate::effects::SlotRef> {
    match g.st.locate(c) {
        Some(ListRef::Slot(p, s)) if g.st.slot_pokemon(p as usize, s) == Some(c) => Some(crate::effects::SlotRef::new(p as usize, s)),
        _ => None,
    }
}

/// What attack `attack` of the Pokémon `attacker` would do now to the opposing Pokémon `target`, at a turn decision of the
/// attacker's player; `g` is not changed.
///
/// - `attacker`: a Pokémon in play of the player whose turn it is. On the Bench, it is put in the Active Spot of the copy
///   the way a scenario arranges a board (a layout change: no ChangeActive event, so no switch consequence and no
///   "moved to the Active Spot this turn" record; the locks are synced to the new layout).
/// - `attack`: an index into the attacks the Pokémon has as the Active Pokémon (`turn::available_attacks`: its printed
///   attacks, then the derived ones: a Tool's, an Ability's grant, a Benched Pokémon's to copy).
/// - `target`: an opposing Pokémon in play.
/// - `extra_energy`: Energy cards attached to the attacker on the copies before the attack, as if already attached: no
///   Attach event (no "when you attach" trigger, no turn's Energy attachment used up). Each is a card of that
///   definition of the attacker's player, taken from their hand, else their deck, else their discard pile (the zone is
///   reported: an attack counting the cards of a zone sees one fewer there). Attached, it provides Energy by the
///   engine's rules for provided Energy (a Special Energy's own rules), and its other rules apply.
/// - Chance: coin `i` of a copy is heads when bit `i` of its script is set. The first copy flips all tails; a copy that
///   flipped `n` coins on a script of length `L` stands for the sequence "script, then tails" (probability `2^-n`) and
///   opens the scripts "the same, heads at coin `j`" for `L <= j < n`, evaluated breadth first, at most `max_runs` copies
///   (at least 1). Every coin of the evaluation counts, either player's.
///
/// Errors: `NOT_A_TURN_DECISION` (the game waits for something else), `INVALID_ATTACKER`, `INVALID_TARGET`,
/// `UNKNOWN_ATTACK`, `UNKNOWN_CARD` / `NOT_AN_ENERGY_CARD` (an `extra_energy` entry), `NO_CARD_TO_ATTACH` (no card of
/// that definition left in the player's hand, deck or discard pile), `CANNOT_BE_ATTACHED` (the card's own rule says it
/// can only be attached to other Pokémon), and any error the attack itself raises (an attack whose text says it can't be
/// used now).
pub fn attack(g: &Game, attacker: CardId, attack: usize, target: CardId, extra_energy: &[DefId], max_runs: usize) -> R<AttackWhatIf> {
    let Pending::Turn(pu) = g.pending() else { return Err(GameError("NOT_A_TURN_DECISION")) };
    let p = pu as usize;
    let from = spot_of(g, attacker).filter(|s| s.p as usize == p).ok_or(GameError("INVALID_ATTACKER"))?;
    let t = spot_of(g, target).filter(|s| s.p as usize == 1 - p).ok_or(GameError("INVALID_TARGET"))?;
    let mut base = g.fork();
    let attacker_moved = from.s != base.st.players[p].active;
    if attacker_moved {
        let bi = base.st.players[p].bench_index_of(from.s).ok_or(GameError("INVALID_ATTACKER"))?;
        let a = base.st.players[p].active;
        base.st.bench_mut(p).as_mut_slice()[bi] = a;
        base.st.set_active(p, from.s);
        base.st.bump_layout();
        crate::spec::passive::lock_sync(&mut base);
    }
    let to = crate::effects::SlotRef::new(p, base.st.players[p].active);
    let extra_attached = attach_extra(&mut base, to, extra_energy)?;
    if !extra_attached.is_empty() {
        crate::spec::passive::lock_sync(&mut base);
    }
    let (atk, copied) = *crate::engine::turn::available_attacks(&mut base, p)?.as_slice().get(attack).ok_or(GameError("UNKNOWN_ATTACK"))?;
    let target_defending = t.s == base.st.players[1 - p].active;
    // The Pokémon in play and the damage taken so far (what the attack's Damage events add to).
    let mut before: Vec<(CardId, i32)> = Vec::new();
    for q in [p, 1 - p] {
        for s in base.st.players[q].in_play() {
            if let Some(c) = base.st.slot_pokemon(q, s) {
                before.push((c, base.st.cards[c as usize].damage_taken_last_turn));
            }
        }
    }
    let ctx = Ctx { p, attack: atk, copied, target, before, prizes: [base.st.players[0].prizes_taken, base.st.players[1].prizes_taken] };
    let mut outcomes: Vec<AttackOutcome> = Vec::new();
    let mut queue: VecDeque<(u32, u32)> = VecDeque::from([(0, 0)]);
    let mut runs = 0;
    while let Some((script, len)) = queue.pop_front() {
        if runs >= max_runs.max(1) {
            break;
        }
        runs += 1;
        let mut c = base.fork();
        c.rng = c.rng.scripted(script);
        let mut o = run(&mut c, &ctx)?;
        let n = c.rng.coins_flipped() as u32;
        o.probability = 0.5f64.powi(n as i32);
        for j in len..n.min(32) {
            queue.push_back((script | 1 << j, j + 1));
        }
        match outcomes.iter_mut().find(|x| AttackOutcome { probability: x.probability, ..o.clone() } == **x) {
            Some(x) => x.probability += o.probability,
            None => outcomes.push(o),
        }
    }
    let explored: f64 = outcomes.iter().map(|o| o.probability).sum();
    Ok(AttackWhatIf { attack: atk, copied, attacker_moved, target_defending, outcomes, runs, unexplored: (1.0 - explored).max(0.0), extra_attached })
}

/// Attaches a card of each definition of `extra` to the Pokémon in `to` on the copy `c`, as if it were already attached:
/// the card is put into the spot (a layout change, as a scenario attaches Energy), with no Attach event. The card is
/// one of the player's own, from the hand, else the deck, else the discard pile.
fn attach_extra(c: &mut Game, to: crate::effects::SlotRef, extra: &[DefId]) -> R<Vec<(DefId, Zone)>> {
    let p = to.p;
    let mut out = Vec::with_capacity(extra.len());
    for &d in extra {
        if d as usize >= crate::carddb::cards().len() {
            return Err(GameError("UNKNOWN_CARD"));
        }
        if crate::carddb::def(d).super_type != SuperType::Energy as u8 {
            return Err(GameError("NOT_AN_ENERGY_CARD"));
        }
        let found = [(ListRef::Hand(p), Zone::Hand), (ListRef::Deck(p), Zone::Deck), (ListRef::Discard(p), Zone::Discard)]
            .into_iter()
            .find_map(|(from, zone)| c.lst(from).iter().copied().find(|&x| c.st.cards[x as usize].def == d).map(|x| (from, zone, x)));
        let Some((from, zone, card)) = found else { return Err(GameError("NO_CARD_TO_ATTACH")) };
        // "This card can only be attached to ...": the card's own rule (its `AttachGuard`, the one the Attach routine checks).
        if crate::spec::passive::attach_guard_refuses(c, card, to)? {
            return Err(GameError("CANNOT_BE_ATTACHED"));
        }
        c.move_card_to(from, card, to.list());
        // The derived facts (Energy provided, HP, ...) are read anew for the next card and the attack.
        c.st.bump_layout();
        out.push((d, zone));
    }
    Ok(out)
}

struct Ctx {
    p: usize,
    attack: AttackRef,
    copied: bool,
    target: CardId,
    before: Vec<(CardId, i32)>,
    prizes: [i32; 2],
}

/// What the attack did when it was over, before the Knock Out check.
struct Done {
    damage: Vec<(CardId, i32)>,
    target_damage: i32,
    target_hp_left: i32,
    to_knock_out: Vec<CardId>,
}

fn done(c: &mut Game, x: &Ctx) -> R<Done> {
    let mut damage = Vec::new();
    let mut target_damage = 0;
    for &(card, d0) in x.before.iter() {
        let d = c.st.cards[card as usize].damage_taken_last_turn - d0;
        if d != 0 {
            damage.push((card, d));
        }
        if card == x.target {
            target_damage = d;
        }
    }
    let target_hp_left = match spot_of(c, x.target) {
        Some(s) => (crate::derived::hp(c, s.p as usize, s.s)? - c.st.slot(s.p as usize, s.s).damage).max(0),
        None => 0,
    };
    let (kos, _) = crate::engine::knockout::find_ko_pokemons(c)?;
    let to_knock_out = kos.iter().filter_map(|s| c.st.slot_pokemon(s.p as usize, s.s)).collect();
    Ok(Done { damage, target_damage, target_hp_left, to_knock_out })
}

/// One evaluation on the copy `c`: the attack, its questions answered with their first valid answer, its chance from the
/// copy's script, then the Knock Out check.
fn run(c: &mut Game, x: &Ctx) -> R<AttackOutcome> {
    crate::engine::turn::use_attack(c, x.p, x.attack, x.copied, true)?;
    let mut asked = Vec::new();
    let mut over: Option<Done> = None;
    let mut settled = false;
    for _ in 0..10_000 {
        match c.pending() {
            Pending::Chance(i) => {
                let r = c.draw_chance(i);
                c.resolve(i, r)?;
            }
            Pending::Info(i) => {
                if over.is_none() && crate::engine::attack::what_if_over(c, i) {
                    over = Some(done(c, x)?);
                }
                c.resolve(i, crate::prompts::Res::True)?;
            }
            Pending::Decision(i) => {
                let pr = c.prompts.as_slice()[i];
                asked.push((pr.player, pr.purpose));
                answer_first(c)?;
            }
            Pending::Finished | Pending::Turn(_) | Pending::Stuck => {
                settled = true;
                break;
            }
        }
    }
    if !settled {
        return Err(GameError("STUCK"));
    }
    // The game ended before the attack was over (no Knock Out check of its own): what it did up to there.
    let d = match over {
        Some(d) => d,
        None => done(c, x)?,
    };
    let knocked_out: Vec<CardId> = d.to_knock_out.iter().copied().filter(|&k| spot_of(c, k).is_none()).collect();
    let prizes = [0, 1].map(|q| (c.st.players[q].prizes_taken - x.prizes[q]).max(0) as u8);
    let winner = if c.st.phase == crate::types::GamePhase::Finished { Some(c.st.winner) } else { None };
    Ok(AttackOutcome {
        probability: 0.0,
        target_knocked_out: knocked_out.contains(&x.target),
        damage: d.damage,
        target_damage: d.target_damage,
        target_hp_left: d.target_hp_left,
        knocked_out,
        prizes,
        winner,
        asked,
    })
}

/// The first valid answer of the open question (the first valid pick, while one is needed).
fn answer_first(c: &mut Game) -> R {
    let sel = c.select()?.ok_or(GameError("STUCK"))?;
    let mut picks = Vec::new();
    loop {
        let (mask, complete) = c.pick_mask(&sel, &picks);
        if picks.len() >= sel.min_count && (complete || picks.len() >= sel.max_count) {
            break;
        }
        match mask.iter().position(|m| *m) {
            Some(k) => picks.push(k),
            None => break,
        }
    }
    c.answer(&sel, &picks)
}

#[cfg(test)]
mod tests {
    //! Expected values come from the card text (data/official_text.json) and the rulebooks: APR A-01 "Damage Calculation
    //! Order" (step 2 the attacker-side effects, 3 Weakness, 4 Resistance, 5 the defender-side effects, 6 "prevents all
    //! damage"), APR II B-08 (Weakness and Resistance for the Active Pokémon only), APR D "Steps for resolving a Knock
    //! Out", the SV rulebook's Appendix 3 (a Pokémon ex Knocked Out gives 2 Prize cards).
    use super::*;
    use serde_json::json;

    const CHARCADET: &str = "Charcadet SCR 29"; // Fire, Flare 30.
    const GROOKEY: &str = "Grookey MEP 52"; // Grass, 70 HP, Weakness Fire; Branch Poke 40.
    const DURALUDON: &str = "Duraludon PRE 69"; // Metal, 130 HP, Weakness Fire, Resistance Grass (-30); Hammer In 30.
    const ARCHALUDON: &str = "Archaludon ex SSP 130"; // Metal, 300 HP, Weakness Fire; Metal Defender 220.
    const EEVEE: &str = "Eevee PRE 74";
    const SYLVEON: &str = "Sylveon PRE 40"; // 120 HP; Safeguard: prevent all damage from the opponent's Pokémon ex; Weakness Metal.
    const BELT: &str = "Maximum Belt PRE 117"; // +50 to the opponent's Active Pokémon ex (before Weakness and Resistance).
    const KANGASKHAN: &str = "Team Rocket's Kangaskhan ex ASC 162"; // Comet Punch: flip 4 coins, 30 damage for each heads.

    fn game(sc: serde_json::Value) -> Game {
        let cards = [(CHARCADET, 2), (GROOKEY, 2), (DURALUDON, 4), (ARCHALUDON, 2), (EEVEE, 1), (SYLVEON, 1), (BELT, 1), (KANGASKHAN, 1)];
        crate::place::tests::game_of(&cards, sc)
    }

    fn me(g: &Game) -> usize {
        g.st.active_player as usize
    }

    fn active(g: &Game, p: usize) -> CardId {
        g.st.active_pokemon(p).unwrap()
    }

    fn bench(g: &Game, p: usize, i: usize) -> CardId {
        g.st.slot_pokemon(p, g.st.players[p].bench.as_slice()[i]).unwrap()
    }

    /// The only outcome of an attack without chance.
    fn sure(w: &AttackWhatIf) -> &AttackOutcome {
        assert_eq!((w.outcomes.len(), w.runs, w.unexplored), (1, 1, 0.0), "{w:?}");
        assert_eq!(w.outcomes[0].probability, 1.0);
        &w.outcomes[0]
    }

    /// Weakness (step 3): Charcadet's Flare 30 against Duraludon (Weakness Fire, x2) does 60; 130 - 60 = 70 HP left.
    /// The caller's game is unchanged.
    #[test]
    fn weakness_doubles_the_damage_to_the_active_pokemon() {
        let g = game(json!({"me": {"reset": true, "active": CHARCADET}, "opp": {"reset": true, "active": DURALUDON}}));
        let (p, hash) = (me(&g), g.state_hash());
        let w = attack(&g, active(&g, p), 0, active(&g, 1 - p), &[], 64).unwrap();
        assert_eq!(g.state_hash(), hash, "the query changed the game");
        assert!(w.target_defending && !w.attacker_moved && !w.copied);
        let o = sure(&w);
        assert_eq!((o.target_damage, o.target_hp_left, o.target_knocked_out, o.prizes, o.winner), (60, 70, false, [0, 0], None));
        assert_eq!(o.damage, vec![(active(&g, 1 - p), 60)]);
        assert!(o.asked.is_empty());
    }

    /// Resistance (step 4): Grookey's Branch Poke 40 against Duraludon (Resistance Grass -30) does 10. As a Benched
    /// attacker it is evaluated as if it were the Active Pokémon: the same 10.
    #[test]
    fn resistance_and_a_benched_attacker() {
        let g = game(json!({"me": {"reset": true, "active": GROOKEY}, "opp": {"reset": true, "active": DURALUDON}}));
        let p = me(&g);
        let o = sure(&attack(&g, active(&g, p), 0, active(&g, 1 - p), &[], 64).unwrap()).clone();
        assert_eq!((o.target_damage, o.target_hp_left), (10, 120));
        let g = game(json!({"me": {"reset": true, "active": CHARCADET, "bench": [{"card": GROOKEY}]}, "opp": {"reset": true, "active": DURALUDON}}));
        let p = me(&g);
        let w = attack(&g, bench(&g, p, 0), 0, active(&g, 1 - p), &[], 64).unwrap();
        assert!(w.attacker_moved);
        assert_eq!(sure(&w).target_damage, 10);
        assert_eq!(g.st.cdef(active(&g, p)).full_name, CHARCADET, "the caller's Active Pokémon is unchanged");
    }

    /// A Benched target: the attack's damage is done to the Defending Pokémon (Grookey, Weakness Fire: 30 x 2 = 60 of its
    /// 70 HP), nothing to the Benched Duraludon (APR II B-08: Flare's damage isn't done to a Benched Pokémon).
    #[test]
    fn a_benched_target_takes_only_what_the_text_does_to_it() {
        let g = game(json!({"me": {"reset": true, "active": CHARCADET}, "opp": {"reset": true, "active": GROOKEY, "bench": [{"card": DURALUDON}]}}));
        let p = me(&g);
        let w = attack(&g, active(&g, p), 0, bench(&g, 1 - p, 0), &[], 64).unwrap();
        assert!(!w.target_defending);
        let o = sure(&w);
        assert_eq!((o.target_damage, o.target_hp_left, o.target_knocked_out), (0, 130, false));
        assert_eq!(o.damage, vec![(active(&g, 1 - p), 60)]);
    }

    /// A Tool (step 2) and a Pokémon ex's Prize cards: Maximum Belt adds 50 before Weakness, (30 + 50) x 2 = 160 to
    /// Archaludon ex; with 150 damage on its 300 HP it is Knocked Out and the attacker takes 2 Prize cards (Appendix 3).
    #[test]
    fn a_tool_modifier_and_a_pokemon_ex_knocked_out() {
        let g = game(json!({"me": {"reset": true, "active": CHARCADET, "active_tool": BELT}, "opp": {"reset": true, "active": [DURALUDON, ARCHALUDON], "active_damage": 150, "bench": [{"card": DURALUDON}]}}));
        let p = me(&g);
        let target = active(&g, 1 - p);
        let o = sure(&attack(&g, active(&g, p), 0, target, &[], 64).unwrap()).clone();
        assert_eq!((o.target_damage, o.target_hp_left), (160, 0));
        assert!(o.target_knocked_out && o.knocked_out == vec![target]);
        let mut pz = [0, 0];
        pz[p] = 2;
        assert_eq!((o.prizes, o.winner), (pz, None));
        // The Prize cards were chosen on the way (the first valid answers).
        assert!(o.asked.contains(&(p as u8, PromptPurpose::TakePrize)), "{:?}", o.asked);
        // Without the Belt: 30 x 2 = 60, no Knock Out.
        let g = game(json!({"me": {"reset": true, "active": CHARCADET}, "opp": {"reset": true, "active": [DURALUDON, ARCHALUDON], "active_damage": 150}}));
        let p = me(&g);
        let o = sure(&attack(&g, active(&g, p), 0, active(&g, 1 - p), &[], 64).unwrap()).clone();
        assert_eq!((o.target_damage, o.target_hp_left, o.target_knocked_out, o.prizes), (60, 90, false, [0, 0]));
    }

    /// "Prevent all damage" (step 6, APR II C-16): Sylveon's Safeguard stops Archaludon ex's Metal Defender (220, x2 for
    /// Sylveon's Metal Weakness) entirely; Duraludon (not a Pokémon ex) does its Hammer In 30 x 2 = 60 of Sylveon's 120 HP.
    #[test]
    fn prevent_all_damage() {
        let g = game(json!({"me": {"reset": true, "active": [DURALUDON, ARCHALUDON], "bench": [{"card": DURALUDON}]}, "opp": {"reset": true, "active": [EEVEE, SYLVEON]}}));
        let p = me(&g);
        let o = sure(&attack(&g, active(&g, p), 0, active(&g, 1 - p), &[], 64).unwrap()).clone();
        assert_eq!((o.target_damage, o.target_hp_left, o.target_knocked_out), (0, 120, false));
        assert!(o.damage.is_empty());
        let o = sure(&attack(&g, bench(&g, p, 0), 0, active(&g, 1 - p), &[], 64).unwrap()).clone();
        assert_eq!((o.target_damage, o.target_hp_left), (60, 60));
    }

    /// Coins: Comet Punch flips 4 coins, 30 damage for each heads, against Duraludon (no Colorless Weakness or
    /// Resistance) with 30 damage on it: 0 / 30 / 60 / 90 / 120 damage with probability 1 / 4 / 6 / 4 / 1 sixteenths;
    /// four heads Knock it Out (30 + 120 >= 130) for 1 Prize card.
    #[test]
    fn coin_flips_are_enumerated() {
        let g = game(json!({"me": {"reset": true, "active": KANGASKHAN}, "opp": {"reset": true, "active": DURALUDON, "active_damage": 30, "bench": [{"card": DURALUDON}]}}));
        let p = me(&g);
        let (a, t) = (active(&g, p), active(&g, 1 - p));
        let w = attack(&g, a, 0, t, &[], 64).unwrap();
        assert_eq!((w.runs, w.unexplored), (16, 0.0));
        let mut got: Vec<(i32, f64, bool, u8)> = w.outcomes.iter().map(|o| (o.target_damage, o.probability * 16.0, o.target_knocked_out, o.prizes[p])).collect();
        got.sort_by_key(|x| x.0);
        assert_eq!(got, vec![(0, 1.0, false, 0), (30, 4.0, false, 0), (60, 6.0, false, 0), (90, 4.0, false, 0), (120, 1.0, true, 1)]);
        // One evaluation: the all-tails sequence alone, the rest unexplored.
        let w = attack(&g, a, 0, t, &[], 1).unwrap();
        assert_eq!((w.runs, w.outcomes.len(), w.outcomes[0].target_damage, w.outcomes[0].probability, w.unexplored), (1, 1, 0, 1.0 / 16.0, 15.0 / 16.0));
    }

    /// The query's arguments are checked: the attacker is the turn player's, the target the opponent's, the attack exists.
    #[test]
    fn arguments_are_checked() {
        let g = game(json!({"me": {"reset": true, "active": CHARCADET}, "opp": {"reset": true, "active": DURALUDON}}));
        let p = me(&g);
        let (a, t) = (active(&g, p), active(&g, 1 - p));
        assert_eq!(attack(&g, t, 0, a, &[], 8).unwrap_err().0, "INVALID_ATTACKER");
        assert_eq!(attack(&g, a, 0, a, &[], 8).unwrap_err().0, "INVALID_TARGET");
        assert_eq!(attack(&g, a, 5, t, &[], 8).unwrap_err().0, "UNKNOWN_ATTACK");
    }

    const FEZANDIPITI: &str = "Fezandipiti PRE 45"; // Psychic Basic; Energy Feather: 30 damage for each Energy attached.
    const MORPEKO: &str = "Marnie's Morpeko DRI 137"; // Darkness Basic; Spiky Wheel 20+: 40 more for each [D] Energy attached.
    const PSYCHIC_ENERGY: &str = "Psychic Energy MEE 5";
    const DARKNESS_ENERGY: &str = "Darkness Energy MEE 7";
    const PRISM: &str = "Prism Energy ASC 216"; // On a Basic Pokémon: every type of Energy, 1 at a time.
    const ROCKET_ENERGY: &str = "Team Rocket's Energy ASC 217"; // Can only be attached to a Team Rocket's Pokémon.

    /// The player's Prize cards are six Metal Energy, so every card listed is in their hand or deck.
    fn energy_game(mut sc: serde_json::Value) -> Game {
        sc["me"]["prizes"] = json!(vec!["Metal Energy MEE 8"; 6]);
        let cards = [(FEZANDIPITI, 1), (MORPEKO, 1), (DURALUDON, 2), (PSYCHIC_ENERGY, 3), (DARKNESS_ENERGY, 2), (PRISM, 1), (ROCKET_ENERGY, 1)];
        crate::place::tests::game_of(&cards, sc)
    }

    fn def(name: &str) -> DefId {
        crate::carddb::def_by_full_name(name).unwrap()
    }

    /// Damage that scales with the attached Energy: Energy Feather does 30 for each Energy attached (card text), so 0 with
    /// none, 60 with two extra Psychic Energy. The cards come from the player's hand first, then the deck; the caller's
    /// game is unchanged. With an empty list the result is the plain query's.
    #[test]
    fn extra_energy_scales_the_damage() {
        let g = energy_game(json!({"me": {"reset": true, "active": FEZANDIPITI, "hand": [PSYCHIC_ENERGY]}, "opp": {"reset": true, "active": DURALUDON}}));
        let (p, hash) = (me(&g), g.state_hash());
        let (a, t) = (active(&g, p), active(&g, 1 - p));
        let base = attack(&g, a, 0, t, &[], 64).unwrap();
        assert_eq!(sure(&base).target_damage, 0);
        assert!(base.extra_attached.is_empty());
        let plain = crate::host::attack_what_if(&g, a, 0, t, 64).unwrap();
        assert_eq!((plain.outcomes, plain.extra_attached), (base.outcomes.clone(), vec![]));
        let psy = def(PSYCHIC_ENERGY);
        let w = attack(&g, a, 0, t, &[psy, psy], 64).unwrap();
        assert_eq!(w.extra_attached, vec![(psy, Zone::Hand), (psy, Zone::Deck)]);
        let o = sure(&w);
        assert_eq!((o.target_damage, o.target_hp_left), (60, 70));
        assert_eq!(g.state_hash(), hash, "the query changed the game");
    }

    /// A Special Energy provides by its own rules: Prism Energy on a Basic Pokémon provides every type of Energy, 1 at a
    /// time (card text), so it counts as one [D] Energy for Spiky Wheel: 20 + 40 = 60, and 20 + 2 x 40 = 100 with a
    /// Darkness Energy too.
    #[test]
    fn a_special_energy_provides_by_its_rules() {
        let g = energy_game(json!({"me": {"reset": true, "active": MORPEKO}, "opp": {"reset": true, "active": DURALUDON}}));
        let p = me(&g);
        let (a, t) = (active(&g, p), active(&g, 1 - p));
        assert_eq!(sure(&attack(&g, a, 0, t, &[], 64).unwrap()).target_damage, 20);
        assert_eq!(sure(&attack(&g, a, 0, t, &[def(PRISM)], 64).unwrap()).target_damage, 60);
        assert_eq!(sure(&attack(&g, a, 0, t, &[def(DARKNESS_ENERGY), def(PRISM)], 64).unwrap()).target_damage, 100);
    }

    /// The extra Energy are checked: a card the player has none of left (two Darkness Energy in the deck, three asked), a
    /// card that isn't an Energy card, an unknown definition, and a card whose own rule forbids this Pokémon (Team
    /// Rocket's Energy "can only be attached to a Team Rocket's Pokémon").
    #[test]
    fn extra_energy_is_checked() {
        let g = energy_game(json!({"me": {"reset": true, "active": MORPEKO}, "opp": {"reset": true, "active": DURALUDON}}));
        let p = me(&g);
        let (a, t) = (active(&g, p), active(&g, 1 - p));
        let dark = def(DARKNESS_ENERGY);
        assert_eq!(attack(&g, a, 0, t, &[dark, dark], 8).unwrap().extra_attached, vec![(dark, Zone::Deck), (dark, Zone::Deck)]);
        assert_eq!(attack(&g, a, 0, t, &[dark, dark, dark], 8).unwrap_err().0, "NO_CARD_TO_ATTACH");
        assert_eq!(attack(&g, a, 0, t, &[def(DURALUDON)], 8).unwrap_err().0, "NOT_AN_ENERGY_CARD");
        assert_eq!(attack(&g, a, 0, t, &[DefId::MAX], 8).unwrap_err().0, "UNKNOWN_CARD");
        assert_eq!(attack(&g, a, 0, t, &[def(ROCKET_ENERGY)], 8).unwrap_err().0, "CANNOT_BE_ATTACHED");
    }

    /// The cost of one query (no assertion; `cargo test --profile iter what_if -- --nocapture` prints it).
    #[test]
    fn cost() {
        let g = game(json!({"me": {"reset": true, "active": KANGASKHAN}, "opp": {"reset": true, "active": DURALUDON}}));
        let (a, t) = (active(&g, me(&g)), active(&g, 1 - me(&g)));
        let g2 = game(json!({"me": {"reset": true, "active": CHARCADET}, "opp": {"reset": true, "active": DURALUDON}}));
        let (a2, t2) = (active(&g2, me(&g2)), active(&g2, 1 - me(&g2)));
        let n = 200;
        let s = std::time::Instant::now();
        for _ in 0..n {
            attack(&g2, a2, 0, t2, &[], 64).unwrap();
        }
        let plain = s.elapsed().as_secs_f64() * 1e6 / n as f64;
        let s = std::time::Instant::now();
        for _ in 0..n {
            attack(&g, a, 0, t, &[], 64).unwrap();
        }
        let coins = s.elapsed().as_secs_f64() * 1e6 / n as f64;
        eprintln!("what_if::attack: {plain:.1} us without chance (1 run), {coins:.1} us for 4 coins (16 runs)");
    }
}
