//! Values: players, zones, slots, numbers, conditions and card predicates
//! (vocabulary v1 "Selectors, predicates and values", "Conditions").
//!
//! Evaluation reads the game and never changes it.

use super::run::Frame;
use crate::effects::SlotRef;
use crate::game::{Game, R};
use crate::list::*;
use crate::prefabs::*;
use crate::state::*;
use crate::types::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Who {
    /// The player whose card is resolving.
    Me,
    Opp,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Zone {
    Deck,
    Hand,
    Discard,
    /// Card register `r` (a scratch list: looked-at cards, chosen cards).
    Scratch(u8),
    /// The Stadium in play on the player's side.
    Stadium,
    /// The cards of the Pokémon's slot: the Pokémon, its Energy and Tools (the player is ignored).
    Attached(SlotExpr),
    /// The Energy cards attached to the Pokémon, as their own list (the player is ignored).
    AttachedEnergy(SlotExpr),
    /// The Pokémon Tools attached to the Pokémon (read-only list; cards move out of the slot's list).
    Tools(SlotExpr),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ZoneRef(pub Who, pub Zone);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotExpr {
    /// The Pokémon this card is (or is attached to).
    This,
    Active(Who),
    /// The Pokémon the last `PickSlot` chose (none when there was nothing to pick).
    Picked,
    /// The Pokémon the program's last attachment went to (none when nothing was attached).
    Attached,
    /// The Pokémon in play this card recorded (`Op::Record`; none when it left play).
    Recorded,
    /// The Pokémon whose attack or Ability caused the event the program reacts to ("the Attacking Pokémon"), where it is
    /// now (none when it left play).
    CausePokemon,
}

pub enum Num {
    Lit(i32),
    ZoneSize(ZoneRef),
    /// Cards in the zone matching the predicate.
    CardCount(ZoneRef, Pred),
    BenchCount(Who),
    OpenBench(Who),
    PrizesLeft(Who),
    DamageOn(SlotExpr),
    Turn,
    Add(&'static Num, &'static Num),
    Sub(&'static Num, &'static Num),
    Mul(&'static Num, &'static Num),
    Min(&'static Num, &'static Num),
    Max(&'static Num, &'static Num),
    /// `if cond { a } else { b }`.
    If(&'static Cond, &'static Num, &'static Num),
    /// Cards with this name in the player's discard pile and on their board.
    KnownCopies(Who, &'static str),
    // --- F-board appends ---
    /// Pokémon selected by the selector that satisfy the slot predicate.
    SlotCount(SlotSel, SlotPred),
    /// Energy on the selected Pokémon (summed). Needs a checked read: only ops
    /// that evaluate with `num_m` can use it.
    EnergyOn(SlotSel, EnergyUnit),
    /// Length of the printed cost of the attack being used.
    PrintedCost,
    // --- F-passive appends ---
    /// Prize cards the player has taken.
    PrizesTaken(Who),
    /// Prize cards the player took during their last turn ("for each Prize card your opponent took during their last turn":
    /// the Prize cards taken during that player's own turn, kept when their turn ended).
    PrizesTakenLastTurn(Who),
    /// Cards in card register `r`.
    RegCount(u8),
    /// Pokémon in play (top card matching the predicate).
    InPlayCount(Who, PlayScope, Pred),
    /// Distinct first provided types among the cards of a zone matching the predicate.
    DistinctTypes(ZoneRef, Pred),
    // --- S3 appends ---
    /// Damage the Pokémon took during the opponent's last turn (the top card's counter).
    DamageTakenLastTurn(SlotExpr),
    /// The cost of the attack being used, as the game checks it now (after cost effects):
    /// a checked read (`num_m`).
    CostNow,
    /// The [C] in the player's Active Pokémon's Retreat Cost as the game checks it: a checked read.
    RetreatCostColorless(Who),
    /// Pokémon Tools attached to the selected Pokémon (summed): "for each Pokémon Tool attached to all of your Pokémon".
    ToolCount(SlotSel),
    /// Heads of the last finished coin sequence.
    Heads,
    /// Total damage (HP) on the selected Pokémon that satisfy the predicate.
    DamageSum(SlotSel, SlotPred),
    /// Cards of the zone matching the predicate, not counting the resolving card
    /// (a Trainer sits in its player's hand while it is checked, and is not in it when used by an attack).
    OthersCount(ZoneRef, Pred),
    /// Cards the program's last discard moved (a discard of chosen Energy).
    Last,
    /// The damage (HP) the attack in progress has done to the opponent's Active Pokémon so far: the amounts of its Damage
    /// events, after Weakness, Resistance and every effect ("heal from this Pokémon the same amount of damage you did to your
    /// opponent's Active Pokémon"; damage a prevention stopped is not done, 0 outside an attack).
    DamageDone,
}

/// Which Pokémon in play a count or condition looks at.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayScope {
    All,
    Bench,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CmpOp {
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,
}

pub enum Cond {
    /// A printed condition of a card's use (a Trainer's "You can use this card only if ...", "You can't use this card during
    /// your first turn"; an Ability's "if this Pokémon is in the Active Spot"): true when the condition is. Unlike the rest
    /// of `needs` (what usability derives), it binds a Trainer whose effect another card uses (Look-Alike Show; user
    /// decision D5; official JP FAQ, Fire Energy MEE 2 / Mela), and the usability sweep doesn't question it.
    Printed(&'static Cond),
    True,
    False,
    Not(&'static Cond),
    All(&'static [Cond]),
    Any(&'static [Cond]),
    Cmp(Num, CmpOp, Num),
    /// The zone holds a card matching the predicate.
    Nonempty(ZoneRef, Pred),
    BenchSpace(Who),
    /// The Pokémon is its player's Active Pokémon.
    IsActive(SlotExpr),
    // --- F-board appends ---
    /// The Pokémon satisfies the slot predicate.
    Slot(SlotExpr, SlotPred),
    /// Some selected Pokémon satisfies the predicate.
    AnySlot(SlotSel, SlotPred),
    /// There is at least one selected Pokémon, and every one satisfies the predicate.
    AllSlots(SlotSel, SlotPred),
    /// Adding these Special Conditions would change the Pokémon.
    WouldChangeConditions(SlotExpr, &'static [SpecialCondition]),
    /// A printed type (of a top Pokémon card) is held by a Pokémon in each selection.
    TypesShared(SlotSel, SlotSel),
    // --- F-passive appends ---
    /// There is a usage record (`Op::Used`): the Ability was used ("once during your turn"), the effect was applied.
    Used(super::ops::state::UsedSpec),
    /// The attack in progress has the flag (`Op::AttackFlag`).
    AttackFlag(super::ops::state::AttackFlagKind),
    /// Exactly `n` cards are in the zone.
    ZoneIs(ZoneRef, i32),
    /// This card is in the player's hand and is the last card there ("You can use this card only if it is the last card in
    /// your hand": Cassiopeia, Gladion's Final Battle). Never for a Trainer whose effect is used from elsewhere (Look-Alike
    /// Show uses it from the opponent's hand: it is never "in your hand"; RULES.md).
    LastCardInHand(Who),
    /// Like `Nonempty`, not counting the resolving card.
    NonemptyOther(ZoneRef, Pred),
    /// Card register `r` holds cards.
    Chosen(u8),
    /// The player played a card matching the predicate from their hand this turn ("if you played a Supporter / Team
    /// Rocket's Supporter / Ancient Supporter card from your hand this turn": the played-this-turn record, APR E-26; a
    /// Supporter's effect used by an attack doesn't count, id2225).
    PlayedThisTurn(Who, Pred),
    /// At least `at_least` cards named `name` in the player's discard pile and in play.
    KnownCopies { who: Who, name: &'static str, at_least: i32 },
    /// A Pokémon in play whose top card matches.
    InPlay(Who, PlayScope, Pred),
    /// A Pokémon in play with any card of its stack matching.
    InPlayAny(Who, PlayScope, Pred),
    // --- S3 agent 3 appends ---
    /// This card's Pokémon moved from the Bench to the Active Spot this turn.
    ThisMovedToActive,
    /// "If you do": the last op that makes events did at least part of what it says (`run::Outcome::Done | Partial`; APR
    /// E-20: having done part of the first half is enough). A refused event, or nothing to do it to, isn't "doing it".
    Done,
    /// "If you can't ... (in full)": the last op that makes events did all it says (`run::Outcome::Done`; APR E-33 specific
    /// case: Lurantis-style "if you can't discard 2 cards in this way, this attack does nothing" negates it).
    DidAll,
    /// The Stadium in play matches the predicate.
    StadiumInPlay(Pred),
    /// Some Basic Pokémon of the player can evolve into a Stage 2 card in the hand, skipping the Stage 1 (a checked read).
    CanEvolveSkippingStage1,
    /// A Pokémon of the player is being Knocked Out by an attack of the other player's Pokémon
    /// and that attacking Pokémon card matches.
    AttackerOfKnockOut { who: Who, pred: Pred },
    /// The player has a Prize card that is still face down and secret.
    FaceDownPrize(Who),
    /// The Trainer being resolved is used as the effect of an attack (Look-Alike Show).
    TrainerViaAttack,
    /// The last Attach op attached at least one card.
    Attached,
    /// During the last turn of the player's opponent, Pokémon of the player were Knocked Out
    /// (by damage from an attack when `by_attack_damage`), one of them carrying `tag`.
    KnockedOutLastTurn { who: Who, by_attack_damage: bool, tag: Option<u32> },
    /// Every name among the Pokémon in play of `names_of` has all 4 copies in zones the owner knows
    /// (own hand, discard pile, Lost Zone and Pokémon in play), so a search for it can't find any.
    AllNamesKnown { names_of: Who },
    /// "If 1 of your [other] [Ancient] Pokémon used [an attack / name] during your last turn" (Koraidon SSP 116, Sylveon ex's
    /// Angelite): the attack the player used during their previous turn (the player's last-attack record: a turn without an
    /// attack, a failed attack and a Knock Out reset leave none) matches `attack` and was used by a Pokémon card matching `by`,
    /// by this card or another card as `attacker` says. A copied attack counts for the copycat that used it (id2402).
    AttackedLastTurn { who: Who, by: Pred, attacker: LastAttacker, attack: AttackPred },
    /// The Ability of this card is blocked for the player the program runs for (a checked read).
    AbilityBlocked,
    /// Some Basic Pokémon the player has in play can evolve now into a card in the game (a checked read).
    CanEvolveBasic(Who),
    /// This Tool's effect is blocked for the player the program runs for (a checked read).
    ToolBlocked,
    /// Some Pokémon of the player in play has an Evolution somewhere in the card pool.
    HasEvolutionInPool(Who),
}

/// Which Pokémon used the attack in a last-attack read (`Cond::AttackedLastTurn`): any Pokémon of the player, this Pokémon
/// ("if this Pokémon used Spiky Rolling during your last turn"), or another one ("if 1 of your other Ancient Pokémon used an
/// attack during your last turn").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LastAttacker {
    Any,
    This,
    Other,
}

/// An attack predicate: which printed attack was used.
pub enum AttackPred {
    Any,
    /// The attack with this printed name (as `CausePred::Attack`).
    Named(&'static str),
}

/// A card predicate.
pub enum Pred {
    Any,
    False,
    Not(&'static Pred),
    All(&'static [Pred]),
    OneOf(&'static [Pred]),
    Pokemon,
    Basic,
    Energy,
    BasicEnergy,
    Trainer,
    Item,
    Supporter,
    Tool,
    Stadium,
    Name(&'static str),
    HpAtMost(i32),
    /// The card has this card tag (`types::tag`).
    Tag(u32),
    /// A Pokémon with at least one attack.
    HasAttacks,
    /// A Pokémon card that prints a Resistance to the type ("Pokémon with [F] Resistance").
    HasResistance(CardType),
    // --- F-board appends ---
    Stage(u8),
    NameContains(&'static str),
    HasAbilityNamed(&'static str),
    HasAttackNamed(&'static str),
    // --- F-passive appends ---
    /// The card prints that it provides the type of Energy.
    ProvidesType(CardType),
    /// The Pokémon card is of this Stage.
    StageIs(Stage),
    /// The Pokémon's printed type includes the type.
    PokemonType(CardType),
    /// Energy card that provides the type.
    Provides(u8),
    // --- S3 agent 3 appends ---
    /// The card has a Rule Box.
    RuleBox,
    /// "Pokémon ex": the lower-case ex rule (Mega Evolution Pokémon ex and Tera Pokémon ex included; not the old "-EX", "-GX").
    PokemonEx,
    /// "Pokémon V": Pokémon V, V-STAR, VMAX and V-UNION (id1667), not Radiant Pokémon.
    PokemonV,
    /// A Pokémon that evolves from the named Pokémon.
    EvolvesFrom(&'static str),
    SpecialEnergy,
    /// The Pokémon evolves from a Pokémon its owner has in play.
    EvolvesFromOwnInPlay,
    /// The Pokémon evolves from its owner's Active Pokémon, by the name the Active Pokémon has now: "a card that evolves
    /// from this Pokémon" in an attack names the Pokémon using the attack, which is a copycat's own name when it uses a
    /// copied attack (JP FAQ Clefable ASC 75: no card evolves from Clefable).
    EvolvesFromActive,
    /// The card prints an Ability (a Pokémon, or a Fossil that is played as one).
    PrintsAbility,
}

impl Frame {
    pub fn who(&self, w: Who) -> usize {
        match w {
            Who::Me => self.p as usize,
            Who::Opp => 1 - self.p as usize,
        }
    }
}

pub fn zone_ref(f: &Frame, z: ZoneRef) -> ListRef {
    let p = f.who(z.0) as u8;
    match z.1 {
        Zone::Deck => ListRef::Deck(p),
        Zone::Hand => ListRef::Hand(p),
        Zone::Discard => ListRef::Discard(p),
        Zone::Scratch(r) => ListRef::Temp(f.cards[r as usize]),
        Zone::Stadium => ListRef::Stadium(p),
        Zone::Attached(_) | Zone::AttachedEnergy(_) | Zone::Tools(_) => panic!("Zone::Attached needs zone_list"),
    }
}

/// The list a zone names (`for_move`: the list cards move out of). None when it names no list
/// (an unset register, a Pokémon that is not in play).
pub fn zone_list(g: &Game, me: CardId, f: &Frame, z: ZoneRef, for_move: bool) -> Option<ListRef> {
    match z.1 {
        Zone::Attached(e) => slot_of(g, me, f, e).map(|s| ListRef::Slot(s.p, s.s)),
        Zone::Tools(e) => slot_of(g, me, f, e).map(|s| ListRef::Slot(s.p, s.s)),
        Zone::AttachedEnergy(e) => slot_of(g, me, f, e).map(|s| if for_move { ListRef::Slot(s.p, s.s) } else { ListRef::SlotEnergies(s.p, s.s) }),
        Zone::Scratch(r) if f.cards[r as usize] == super::run::NONE => None,
        _ => Some(zone_ref(f, z)),
    }
}

/// The cards of one zone, copied out without a heap list (the longest list is a scratch list, `Game::temps`).
pub type ZoneCards = SVec<CardId, 120>;

/// The cards of a zone (empty when it names no list).
pub fn zone_cards_of(g: &Game, me: CardId, f: &Frame, z: ZoneRef) -> ZoneCards {
    let mut out = ZoneCards::new();
    if let Zone::Tools(e) = z.1 {
        if let Some(s) = slot_of(g, me, f, e) {
            out.extend_from_slice(g.st.slot(s.p as usize, s.s).tools().as_slice());
        }
        return out;
    }
    match zone_list(g, me, f, z, false) {
        // A legality check's played Item and its companion are out of the hand (`Frame::excl`).
        Some(l @ ListRef::Hand(_)) if f.excl[0] != crate::list::NO_CARD => {
            for &c in g.lst(l) {
                if !f.out_of_hand(c) {
                    out.push(c);
                }
            }
        }
        Some(l) => out.extend_from_slice(g.lst(l)),
        None => {}
    }
    out
}

pub fn slot_of(g: &Game, me: CardId, f: &Frame, s: SlotExpr) -> Option<SlotRef> {
    match s {
        SlotExpr::Attached => (f.attached_to != super::run::NONE).then(|| SlotRef::new((f.attached_to >> 4) as usize, f.attached_to & 15)),
        SlotExpr::Picked => (f.slot != super::run::NONE).then(|| SlotRef::new((f.slot >> 4) as usize, f.slot & 15)),
        SlotExpr::Active(w) => {
            let p = f.who(w);
            Some(SlotRef::new(p, g.st.players[p].active))
        }
        SlotExpr::CausePokemon => {
            if f.eff as usize >= g.fx.len() {
                return None;
            }
            let c = g.e(f.eff).cause()?.card?;
            let (q, s) = g.st.find_pokemon_slot(c)?;
            (g.st.slot_pokemon(q, s) == Some(c)).then(|| SlotRef::new(q, s))
        }
        SlotExpr::Recorded => crate::engine::lasting::recorded_slot(&g.st, me).map(|(p, s)| SlotRef::new(p, s)),
        SlotExpr::This => {
            for p in 0..2 {
                let pl = &g.st.players[p];
                let mut slots: SVec<SlotId, 9> = SVec::new();
                slots.push(pl.active);
                for b in pl.bench.iter() {
                    slots.push(*b);
                }
                for s in slots.iter().copied() {
                    let sl = &pl.slots[s as usize];
                    if sl.cards().contains(me) || sl.tools().contains(me) {
                        return Some(SlotRef::new(p, s));
                    }
                }
            }
            None
        }
    }
}

pub fn num(g: &Game, me: CardId, f: &Frame, n: &Num) -> i32 {
    match n {
        Num::Lit(v) => *v,
        Num::ZoneSize(z) => zone_cards_of(g, me, f, *z).len() as i32,
        Num::CardCount(z, p) => zone_cards_of(g, me, f, *z).iter().filter(|c| pred(g, **c, p)).count() as i32,
        Num::BenchCount(w) => {
            let pl = &g.st.players[f.who(*w)];
            pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards().is_empty()).count() as i32
        }
        Num::OpenBench(w) => empty_bench_slots(g, f.who(*w)).len() as i32,
        Num::PrizesLeft(w) => g.st.players[f.who(*w)].prize_left() as i32,
        Num::DamageOn(s) => slot_of(g, me, f, *s).map(|s| g.st.slot(s.p as usize, s.s).damage).unwrap_or(0),
        Num::Turn => g.st.turn as i32,
        Num::Add(a, b) => num(g, me, f, a) + num(g, me, f, b),
        Num::Sub(a, b) => num(g, me, f, a) - num(g, me, f, b),
        Num::Mul(a, b) => num(g, me, f, a) * num(g, me, f, b),
        Num::Min(a, b) => num(g, me, f, a).min(num(g, me, f, b)),
        Num::Max(a, b) => num(g, me, f, a).max(num(g, me, f, b)),
        Num::If(c, a, b) => {
            if cond(g, me, f, c) {
                num(g, me, f, a)
            } else {
                num(g, me, f, b)
            }
        }
        Num::KnownCopies(w, name) => {
            let p = f.who(*w);
            let mut n = g.st.players[p].discard().iter().filter(|c| g.st.cdef(*c).name == *name).count();
            for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter() {
                n += g.st.slot(p, *s).cards().iter().filter(|c| g.st.cdef(*c).name == *name).count();
            }
            n as i32
        }
        Num::SlotCount(sel, sp) => slots_of(g, me, f, sel).iter().filter(|s| slot_pred(g, me, **s, sp).expect("checked read: evaluate with num_m")).count() as i32,
        Num::EnergyOn(..) => panic!("Num::EnergyOn needs a checked read (num_m)"),
        Num::PrintedCost => printed_cost(g, f),
        Num::PrizesTaken(w) => 6 - g.st.players[f.who(*w)].prize_left() as i32,
        Num::PrizesTakenLastTurn(w) => g.st.players[f.who(*w)].prizes_taken_last_turn,
        Num::RegCount(r) => reg_list(g, f, *r).len() as i32,
        Num::InPlayCount(w, scope, p) => in_play(g, f.who(*w), *scope).filter(|(_, top, _)| pred(g, *top, p)).count() as i32,
        Num::Heads => f.heads as i32,
        Num::DamageTakenLastTurn(s) => slot_of(g, me, f, *s).and_then(|s| g.st.slot_pokemon(s.p as usize, s.s)).map(|c| g.st.cards[c as usize].damage_taken_last_turn).unwrap_or(0),
        Num::CostNow => panic!("Num::CostNow needs a checked read (num_m)"),
        Num::ToolCount(sel) => slots_of(g, me, f, sel).iter().map(|s| g.st.slot(s.p as usize, s.s).tools().len() as i32).sum(),
        Num::RetreatCostColorless(_) => panic!("Num::RetreatCostColorless needs a checked read (num_m)"),
        Num::DamageSum(sel, sp) => slots_of(g, me, f, sel).iter().filter(|s| slot_pred(g, me, **s, sp).unwrap_or(false)).map(|s| g.st.slot(s.p as usize, s.s).damage).sum(),
        Num::Last => f.last,
        Num::DamageDone => crate::spec::ops::board::frame_attack(g, f).and_then(|_| g.last_attack.as_ref()).map_or(0, |la| la.damage_done),
        Num::OthersCount(z, p) => zone_cards_of(g, me, f, *z).iter().filter(|c| **c != me && pred(g, **c, p)).count() as i32,
        Num::DistinctTypes(z, p) => {
            let mut types: Vec<u8> = Vec::new();
            for c in g.lst(zone_ref(f, *z)).iter() {
                let d = g.st.cdef(*c);
                if !f.out_of_hand(*c) && pred(g, *c, p) {
                    if let Some(t) = d.provides.first() {
                        if !types.contains(t) {
                            types.push(*t);
                        }
                    }
                }
            }
            types.len() as i32
        }
    }
}

/// The cards of card register `r` (empty while it is unset).
pub fn reg_list<'a>(g: &'a Game, f: &Frame, r: u8) -> &'a [CardId] {
    match f.cards[r as usize] {
        super::run::NONE => &[],
        i => g.lst(ListRef::Temp(i)),
    }
}

/// Pokémon in play, Active first: (slot, top card, stack).
pub fn in_play(g: &Game, p: usize, scope: PlayScope) -> impl Iterator<Item = (SlotId, CardId, &[CardId])> + '_ {
    let pl = &g.st.players[p];
    let active = (scope == PlayScope::All).then_some(pl.active);
    active.into_iter().chain(pl.bench.iter().copied()).filter_map(move |s| g.st.slot_pokemon(p, s).map(|top| (s, top, pl.slots[s as usize].cards().as_slice())))
}

pub fn cond(g: &Game, me: CardId, f: &Frame, c: &Cond) -> bool {
    match c {
        Cond::True => true,
        Cond::False => false,
        Cond::Not(c) => !cond(g, me, f, c),
        Cond::All(cs) => cs.iter().all(|c| cond(g, me, f, c)),
        Cond::Any(cs) => cs.iter().any(|c| cond(g, me, f, c)),
        Cond::Cmp(a, op, b) => {
            let (a, b) = (num(g, me, f, a), num(g, me, f, b));
            match op {
                CmpOp::Lt => a < b,
                CmpOp::Le => a <= b,
                CmpOp::Eq => a == b,
                CmpOp::Ne => a != b,
                CmpOp::Ge => a >= b,
                CmpOp::Gt => a > b,
            }
        }
        Cond::Nonempty(z, p) => zone_cards_of(g, me, f, *z).iter().any(|c| pred(g, *c, p)),
        Cond::BenchSpace(w) => !empty_bench_slots(g, f.who(*w)).is_empty(),
        Cond::IsActive(s) => slot_of(g, me, f, *s).map(|s| g.st.players[s.p as usize].active == s.s).unwrap_or(false),
        Cond::Slot(e, sp) => match slot_of(g, me, f, *e) {
            Some(s) => slot_pred(g, me, s, sp).expect("checked read: evaluate with cond_m"),
            None => false,
        },
        Cond::AnySlot(sel, sp) => slots_of(g, me, f, sel).iter().any(|s| slot_pred(g, me, *s, sp).expect("checked read: evaluate with cond_m")),
        Cond::AllSlots(sel, sp) => {
            let v = slots_of(g, me, f, sel);
            !v.is_empty() && v.iter().all(|s| slot_pred(g, me, *s, sp).expect("checked read: evaluate with cond_m"))
        }
        Cond::WouldChangeConditions(e, cs) => match slot_of(g, me, f, *e) {
            Some(s) => crate::engine::phase::would_change_special_conditions(g.st.slot(s.p as usize, s.s), cs),
            None => false,
        },
        Cond::Used(u) => super::ops::state::is_used(g, me, f, u),
        Cond::AttackFlag(k) => (f.eff as usize) < g.fx.len() && matches!(k, super::ops::state::AttackFlagKind::OptionChosen) && g.fx_flags(f.eff) & crate::game::fx_flag::OPTION_CHOSEN != 0,
        Cond::ZoneIs(z, n) => zone_cards_of(g, me, f, *z).len() as i32 == *n,
        Cond::LastCardInHand(w) => {
            let hand = &g.st.players[f.who(*w)].hand();
            !f.out_of_hand(me) && hand.contains(me) && hand.iter().filter(|c| !f.out_of_hand(*c)).all(|c| c == me)
        }
        Cond::TypesShared(..) => panic!("checked read: evaluate with cond_m"),
        Cond::NonemptyOther(z, p) => zone_cards_of(g, me, f, *z).iter().any(|c| *c != me && pred(g, *c, p)),
        Cond::Chosen(r) => !reg_list(g, f, *r).is_empty(),
        Cond::PlayedThisTurn(w, pr) => g.st.players[f.who(*w)].played_this_turn.iter().any(|c| pred(g, *c, pr)),
        Cond::KnownCopies { who, name, at_least } => {
            let p = f.who(*who);
            let mut n = g.lst(ListRef::Discard(p as u8)).iter().filter(|c| g.st.cdef(**c).name == *name).count() as i32;
            for (_, _, stack) in in_play(g, p, PlayScope::All) {
                n += stack.iter().filter(|c| g.st.cdef(**c).name == *name).count() as i32;
            }
            n >= *at_least
        }
        Cond::InPlay(w, scope, p) => in_play(g, f.who(*w), *scope).any(|(_, top, _)| pred(g, top, p)),
        Cond::InPlayAny(w, scope, p) => in_play(g, f.who(*w), *scope).any(|(_, _, stack)| stack.iter().any(|c| pred(g, *c, p))),
        Cond::ThisMovedToActive => g.st.players[f.p as usize].moved_to_active_this_turn.contains(&me),
        Cond::Done => f.outcome.did(),
        Cond::DidAll => f.outcome == super::run::Outcome::Done,
        Cond::Printed(c) => cond(g, me, f, c),
        Cond::StadiumInPlay(p) => g.st.stadium_card().map(|c| pred(g, c, p)).unwrap_or(false),
        Cond::CanEvolveSkippingStage1 => panic!("Cond::CanEvolveSkippingStage1 needs a checked read (cond_m)"),
        Cond::TrainerViaAttack => f.via_attack,
        Cond::Attached => f.attached_to != super::run::NONE,
        Cond::KnockedOutLastTurn { who, by_attack_damage, tag } => {
            let pl = &g.st.players[f.who(*who)];
            // Each Pokémon's own cause counts: "any of your X Pokémon were Knocked Out by damage from an attack".
            pl.pokemon_knocked_out_last_turn_entries.iter().enumerate().any(|(i, d)| {
                (!*by_attack_damage || pl.pokemon_knocked_out_last_turn_by_attack.as_slice().get(i).copied().unwrap_or(false))
                    && tag.map_or(true, |t| crate::carddb::def(*d).has_tag(t))
            })
        }
        Cond::FaceDownPrize(w) => {
            let pl = &g.st.players[f.who(*w)];
            (0..pl.prize_count as usize).any(|i| !pl.prize_public[i] && !pl.prize_face_up[i] && !pl.prizes()[i].is_empty())
        }
        Cond::AttackerOfKnockOut { who, pred: q } => g.attacker_of_knock_out(f.who(*who)).and_then(|(c, _)| c).map_or(false, |c| pred(g, c, q)),
        Cond::AbilityBlocked => panic!("Cond::AbilityBlocked needs a checked read (cond_m)"),
        Cond::CanEvolveBasic(_) => panic!("Cond::CanEvolveBasic needs a checked read (cond_m)"),
        Cond::ToolBlocked => panic!("Cond::ToolBlocked needs a checked read (cond_m)"),
        Cond::AttackedLastTurn { who, by, attacker, attack } => {
            // The player's previous turn is two turns ago (turns alternate: no card in the pool skips a turn); an attack
            // that failed (Confusion) isn't used (its stamp is voided).
            let p = f.who(*who);
            match g.st.player_last_attack[p] {
                Some((a, src)) => {
                    g.st.player_last_attack_turn[p] == g.st.turn - 2
                        && match attacker {
                            LastAttacker::Any => true,
                            LastAttacker::This => src == me,
                            LastAttacker::Other => src != me,
                        }
                        && pred(g, src, by)
                        && match attack {
                            AttackPred::Any => true,
                            AttackPred::Named(name) => crate::engine::attack::attack_def(g, a).name == *name,
                        }
                }
                None => false,
            }
        }
        Cond::AllNamesKnown { names_of } => {
            let p = f.who(Who::Me);
            let pl = &g.st.players[p];
            let known = |name: &str| -> usize {
                let named = |c: CardId| {
                    let d = g.st.cdef(c);
                    d.is_pokemon() && d.name == name
                };
                let mut n = pl.hand().iter().filter(|c| named(*c)).count() + pl.discard().iter().filter(|c| named(*c)).count() + pl.lostzone().iter().filter(|c| named(*c)).count();
                for s in pl.in_play() {
                    n += pl.slots[s as usize].cards().iter().filter(|c| named(*c)).count();
                }
                n
            };
            in_play(g, f.who(*names_of), PlayScope::All).all(|(_, top, _)| known(g.st.cdef(top).name) >= 4)
        }
        Cond::HasEvolutionInPool(w) => {
            let p = f.who(*w);
            for (_, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
                let base = g.st.cdef(c).name;
                if crate::gen::evolutions::ALL_EVOLUTIONS.iter().any(|(_, from)| *from == base) {
                    return true;
                }
            }
            false
        }
    }
}

/// The Fossil is the Pokémon of one of its owner's slots.
fn fossil_in_play(g: &Game, c: CardId) -> bool {
    let p = g.st.owner(c);
    g.st.players[p].in_play().any(|s| g.st.slot_pokemon(p, s) == Some(c))
}

/// Does the card satisfy the predicate? Pure: it reads the card's definition and, for a Fossil and the in-play variants,
/// the board.
pub fn pred(g: &Game, c: CardId, p: &Pred) -> bool {
    pred_def(g, c, g.st.cdef(c), p)
}

/// `pred` with the card's definition `d` read once for the whole tree.
fn pred_def(g: &Game, c: CardId, d: &'static crate::carddb::CardDef, p: &Pred) -> bool {
    // A Fossil in play is a Basic Pokémon (its card data carries the stage, type and HP); asked only by the variants that
    // read it.
    let mon = || d.is_pokemon() || (crate::carddb::pokemon_flags(g.st.cards[c as usize].def) & crate::carddb::PF_FOSSIL != 0 && fossil_in_play(g, c));
    match p {
        Pred::Any => true,
        Pred::False => false,
        Pred::Not(p) => !pred_def(g, c, d, p),
        Pred::All(ps) => ps.iter().all(|p| pred_def(g, c, d, p)),
        Pred::OneOf(ps) => ps.iter().any(|p| pred_def(g, c, d, p)),
        Pred::Pokemon => mon(),
        Pred::Basic => mon() && d.stage == Stage::Basic as u8,
        Pred::Energy => d.is_energy(),
        Pred::BasicEnergy => d.is_energy() && d.energy_type == EnergyType::Basic as u8,
        Pred::Trainer => d.is_trainer(),
        Pred::Item => d.is_trainer() && d.trainer_type == TrainerType::Item as u8,
        Pred::Supporter => d.is_trainer() && d.trainer_type == TrainerType::Supporter as u8,
        Pred::Tool => d.is_trainer() && d.trainer_type == TrainerType::Tool as u8,
        Pred::Stadium => d.is_trainer() && d.trainer_type == TrainerType::Stadium as u8,
        Pred::Name(n) => d.name == *n,
        Pred::HpAtMost(n) => mon() && d.hp <= *n,
        Pred::Tag(t) => d.has_tag(*t),
        Pred::HasAttacks => mon() && !d.attacks.is_empty(),
        Pred::HasResistance(t) => mon() && d.resistance.iter().any(|r| r.card_type == *t),
        Pred::Stage(st) => mon() && d.stage == *st,
        Pred::NameContains(n) => d.name.contains(*n),
        Pred::HasAbilityNamed(n) => mon() && d.powers.iter().any(|p| p.source() == crate::spec::passive::RuleSource::Ability && p.name == *n),
        Pred::HasAttackNamed(n) => d.attacks.iter().any(|a| a.name == *n),
        Pred::ProvidesType(t) => d.provides.contains(t),
        Pred::StageIs(st) => mon() && d.stage == *st as u8,
        Pred::PokemonType(t) => mon() && d.card_type.contains(t),
        Pred::Provides(t) => d.is_energy() && d.provides.contains(t),
        Pred::RuleBox => d.has_rule_box(),
        Pred::PokemonEx => d.is_pokemon_ex(),
        Pred::PokemonV => d.is_pokemon_v(),
        Pred::EvolvesFrom(n) => mon() && d.evolves_from == *n,
        Pred::SpecialEnergy => d.is_energy() && d.energy_type == EnergyType::Special as u8,
        Pred::EvolvesFromOwnInPlay => {
            mon() && {
                let owner = g.st.owner(c);
                for_each_pokemon(g, owner, PlayerType::BottomPlayer).iter().any(|(_, top, _)| g.st.cdef(*top).name == d.evolves_from)
            }
        }
        Pred::EvolvesFromActive => {
            mon() && {
                let owner = g.st.owner(c);
                let active = g.st.players[owner].active;
                g.st.slot_pokemon(owner, active).is_some_and(|top| g.st.cdef(top).name == d.evolves_from)
            }
        }
        Pred::PrintsAbility => d.powers.iter().any(|pw| pw.source() == crate::spec::passive::RuleSource::Ability),
    }
}

// ---------------------------------------------------------------------------
// F-board appends: slot selectors, slot predicates, checked reads

/// A collection of Pokémon in play.
pub enum SlotSel {
    One(SlotExpr),
    /// The occupied Bench slots of a player.
    Bench(Who),
    /// Every Pokémon of a player: the Active, then the Bench.
    Pokemon(Who),
    Filtered(&'static SlotSel, SlotPred),
    /// Every Pokémon of a player; a prompt lists the Bench before the Active Spot.
    PokemonBenchFirst(Who),
    // --- S3-4 appends ---
    /// The same Pokémon, but the prompt that chooses among them can be cancelled.
    Cancelable(&'static SlotSel),
}

/// A predicate on a Pokémon in play.
pub enum SlotPred {
    Any,
    Not(&'static SlotPred),
    All(&'static [SlotPred]),
    OneOf(&'static [SlotPred]),
    /// Has at least one damage counter.
    Damaged,
    IsActive,
    IsBench,
    /// The top Pokémon card satisfies the card predicate.
    Top(Pred),
    /// Is affected by any Special Condition.
    HasCondition,
    /// The type of the Pokémon as the game checks it (Energy and Stadiums can
    /// change it): a checked read.
    TypeIs(CardType),
    /// The Pokémon card directly under this card in the stack has this name.
    CardBelowThis(&'static str),
    /// The Pokémon came into play this turn: put into play, evolved or devolved (`Slot::entered_turn`).
    PlayedThisTurn,
    // --- F-passive appends ---
    /// The slot this card is part of (the Pokémon it is, or the one it is attached to).
    Holder,
    /// The top Pokémon carries the tag.
    Tag(u32),
    /// The top Pokémon has a Rule Box (a Pokémon in play is its top card: a Rule Box of a card under it isn't the
    /// Pokémon's).
    RuleBox,
    /// The Pokémon's Energy provides the type (any-type Energy counts): a checked read.
    Provides(CardType),
    /// The top card is a Basic Pokémon.
    Basic,
    /// The top card is of this Stage.
    StageIs(Stage),
    /// The top Pokémon evolves from another (an empty slot counts as one).
    Evolution,
    /// The Pokémon has an Ability after effects: a checked read.
    HasAbility,
    /// The Pokémon has the Ability of this name after effects ("if your Active Pokémon has the Festival Lead Ability": a
    /// locked Ability isn't had, JP FAQ Thwackey / Flutter Mane; RULES.md "Pokémon that have an Ability"): a checked read.
    /// `Pred::HasAbilityNamed` stays the printed read for cards out of play.
    HasAbilityNamed(&'static str),
    /// The top Pokémon is this card.
    IsThisPokemon,
    /// The top Pokémon has this name.
    Named(&'static str),
    /// The Pokémon's Energy provides nothing: a checked read.
    NoEnergyProvided,
    /// The slot has an Energy card attached.
    HasEnergy,
    /// The Pokémon's remaining HP (with effects) is at most this much: a checked read.
    RemainingHpAtMost(i32),
    /// Some card of the slot (the Pokémon, attached Energy) matches the predicate.
    HasCard(Pred),
    /// The Pokémon is affected by this Special Condition.
    Condition(SpecialCondition),
    /// This card recorded the Pokémon (`Op::Record`).
    Recorded,
    /// The Pokémon has a Special Energy card attached.
    HasSpecialEnergy,
    /// A Pokémon Tool attached to the slot matches the card predicate.
    AnyTool(Pred),
    /// An Energy card with this name is attached.
    HasEnergyNamed(&'static str),
    /// The Pokémon is evolved.
    Evolved,
    /// The Pokémon is on the side of the player who owns this card.
    OnMySide,
    /// Damage counters were removed from the Pokémon during this turn ("if this Pokémon was healed during this turn": a
    /// heal that removed nothing isn't healing, id2198; `Slot::healed_this_turn`).
    HealedThisTurn,
}

impl SlotPred {
    /// Can the answer depend on the cards attached to the Pokémon (the attached cards themselves, or a checked
    /// read they can change: type, Energy provided, HP, Abilities)? Conservative: a
    /// predicate not known to read only the Pokémon card, its position, counters, conditions and records says
    /// yes.
    pub const fn reads_attached(&self) -> bool {
        match self {
            SlotPred::Not(p) => p.reads_attached(),
            SlotPred::All(ps) | SlotPred::OneOf(ps) => {
                let mut i = 0;
                while i < ps.len() {
                    if ps[i].reads_attached() {
                        return true;
                    }
                    i += 1;
                }
                false
            }
            SlotPred::Any
            | SlotPred::Damaged
            | SlotPred::IsActive
            | SlotPred::IsBench
            | SlotPred::Top(_)
            | SlotPred::HasCondition
            | SlotPred::CardBelowThis(_)
            | SlotPred::PlayedThisTurn
            | SlotPred::HealedThisTurn
            | SlotPred::Tag(_)
            | SlotPred::Basic
            | SlotPred::StageIs(_)
            | SlotPred::Evolution
            | SlotPred::IsThisPokemon
            | SlotPred::Named(_)
            | SlotPred::Condition(_)
            | SlotPred::Recorded
            | SlotPred::Evolved
            | SlotPred::OnMySide => false,
            _ => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnergyUnit {
    /// Attached Special Energy cards. Pure.
    SpecialEnergyCards,
    /// Provided Energy: the `provides` entries equal to the type or to ANY
    /// (CheckProvidedEnergy).
    Provided(CardType),
    /// Every `provides` entry of every Energy card (CheckProvidedEnergy).
    ProvidedUnits,
    /// Energy cards (map entries) that provide the type, or every type: a text that says "Energy cards" (a count of
    /// Energy attached is the Energy provided otherwise: user decision 2026-10-12, id2237).
    MatchingEntries(CardType),
}

/// Printed cost length of the attack being used.
fn printed_cost(g: &Game, f: &Frame) -> i32 {
    match crate::prefabs::attack_data(g, f.eff) {
        Some((_, _, a, _)) => crate::engine::attack::attack_def(g, a).cost.len() as i32,
        None => 0,
    }
}

pub fn slots_of(g: &Game, me: CardId, f: &Frame, sel: &SlotSel) -> SVec<SlotRef, { crate::state::MAX_SLOT_REFS }> {
    let mut out: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }> = SVec::new();
    match sel {
        SlotSel::One(e) => {
            if let Some(s) = slot_of(g, me, f, *e) {
                if !g.st.slot(s.p as usize, s.s).cards().is_empty() {
                    out.push(s);
                }
            }
        }
        SlotSel::Bench(w) => {
            let p = f.who(*w);
            let pl = &g.st.players[p];
            for b in pl.bench.iter() {
                if !pl.slots[*b as usize].cards().is_empty() {
                    out.push(SlotRef::new(p, *b));
                }
            }
        }
        SlotSel::Pokemon(w) | SlotSel::PokemonBenchFirst(w) => {
            let p = f.who(*w);
            for s in g.st.players[p].in_play() {
                out.push(SlotRef::new(p, s));
            }
        }
        SlotSel::Filtered(inner, sp) => {
            for s in slots_of(g, me, f, inner).iter() {
                if slot_pred(g, me, *s, sp).unwrap_or(true) {
                    out.push(*s);
                }
            }
        }
        SlotSel::Cancelable(inner) => return slots_of(g, me, f, inner),
    }
    out
}

/// The pure reading of a slot predicate; `None` when it needs a checked read.
pub fn slot_pred(g: &Game, me: CardId, s: SlotRef, sp: &SlotPred) -> Option<bool> {
    let (p, id) = (s.p as usize, s.s);
    let slot = g.st.slot(p, id);
    Some(match sp {
        SlotPred::Any => true,
        SlotPred::Not(q) => !slot_pred(g, me, s, q)?,
        SlotPred::All(qs) => {
            let mut r = true;
            for q in qs.iter() {
                r &= slot_pred(g, me, s, q)?;
            }
            r
        }
        SlotPred::OneOf(qs) => {
            let mut r = false;
            for q in qs.iter() {
                r |= slot_pred(g, me, s, q)?;
            }
            r
        }
        SlotPred::Damaged => slot.damage > 0,
        SlotPred::IsActive => g.st.players[p].active == id,
        SlotPred::IsBench => g.st.players[p].active != id,
        SlotPred::Top(q) => g.st.slot_pokemon(p, id).map(|c| pred(g, c, q)).unwrap_or(false),
        SlotPred::HasCondition => !slot.special_conditions.is_empty(),
        SlotPred::TypeIs(_) => return None,
        SlotPred::CardBelowThis(name) => {
            let stack = g.st.slot_pokemons(p, id);
            stack.iter().position(|c| *c == me).and_then(|i| i.checked_sub(1)).map_or(false, |i| g.st.cdef(stack.as_slice()[i]).name == *name)
        }
        SlotPred::PlayedThisTurn => slot.entered_turn == g.st.turn as i32,
        SlotPred::HealedThisTurn => slot.healed_this_turn,
        SlotPred::Holder => slot.cards().contains(me) || slot.tools().contains(me),
        SlotPred::Tag(t) => g.st.slot_pokemon(p, id).map(|c| g.st.cdef(c).has_tag(*t)).unwrap_or(false),
        SlotPred::RuleBox => g.st.slot_pokemon(p, id).map(|c| g.st.cdef(c).has_rule_box()).unwrap_or(false),
        SlotPred::Basic => g.st.slot_pokemon(p, id).map(|c| g.st.cdef(c).stage == Stage::Basic as u8).unwrap_or(false),
        SlotPred::StageIs(st) => g.st.slot_pokemon(p, id).map(|c| g.st.cdef(c).stage == *st as u8).unwrap_or(false),
        SlotPred::Evolution => g.st.slot_pokemon(p, id).map(|c| !g.st.cdef(c).evolves_from.is_empty()).unwrap_or(true),
        SlotPred::IsThisPokemon => g.st.slot_pokemon(p, id) == Some(me),
        SlotPred::Named(n) => g.st.slot_pokemon(p, id).map(|c| g.st.cdef(c).name == *n).unwrap_or(false),
        SlotPred::HasEnergy => !slot.energies().is_empty(),
        SlotPred::HasCard(q) => slot.cards().iter().any(|c| pred(g, c, q)),
        SlotPred::Condition(c) => slot.special_conditions.contains(&(*c as u8)),
        SlotPred::AnyTool(q) => slot.tools().iter().any(|c| pred(g, c, q)),
        SlotPred::HasSpecialEnergy => slot.energies().iter().any(|c| g.st.cdef(c).energy_type == EnergyType::Special as u8),
        SlotPred::Recorded => crate::engine::lasting::is_recorded(&g.st, p, id, me),
        SlotPred::Provides(_) | SlotPred::HasAbility | SlotPred::HasAbilityNamed(_) | SlotPred::NoEnergyProvided | SlotPred::RemainingHpAtMost(_) => return None,
        SlotPred::HasEnergyNamed(n) => slot.cards().iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.name == *n
        }),
        SlotPred::OnMySide => {
            let owner = g.st.locate(me).and_then(|l| l.owner()).unwrap_or_else(|| g.st.owner(me));
            p == owner
        }
        SlotPred::Evolved => {
            let stack = g.st.slot_pokemons(p, id);
            if stack.len() <= 1 {
                false
            } else {
                match g.st.slot_pokemon(p, id) {
                    Some(top) => {
                        let st = g.st.cdef(top).stage;
                        !(st == Stage::Legend as u8 || st == Stage::Vunion as u8 || (st == Stage::LvX as u8 && stack.len() == 2))
                    }
                    None => true,
                }
            }
        }
    })
}

/// Slot predicate with checked reads.
pub fn slot_pred_m<F: crate::derived::Facts>(g: &mut F, me: CardId, s: SlotRef, sp: &SlotPred) -> R<bool> {
    Ok(match sp {
        SlotPred::Not(q) => !slot_pred_m(g, me, s, q)?,
        SlotPred::All(qs) => {
            for q in qs.iter() {
                if !slot_pred_m(g, me, s, q)? {
                    return Ok(false);
                }
            }
            true
        }
        SlotPred::OneOf(qs) => {
            for q in qs.iter() {
                if slot_pred_m(g, me, s, q)? {
                    return Ok(true);
                }
            }
            false
        }
        SlotPred::TypeIs(t) => g.types(s)?.contains(t),
        SlotPred::Provides(t) => g.provided_energy(s.p as usize, s)?.iter().any(|m| m.provides.contains(t) || m.provides.contains(&ct::ANY)),
        SlotPred::NoEnergyProvided => g.provided_energy(s.p as usize, s)?.is_empty(),
        SlotPred::RemainingHpAtMost(n) => {
            if g.g().st.slot_pokemon(s.p as usize, s.s).is_none() {
                return Ok(false);
            }
            let hp = g.hp(s.p as usize, s.s)?;
            hp - g.g().st.slot(s.p as usize, s.s).damage <= *n
        }
        SlotPred::HasAbility => {
            let Some(src) = g.g().st.slot_pokemon(s.p as usize, s.s) else { return Ok(false) };
            g.powers(src)?.iter().any(|r| g.g().st.cdef(r.card).powers[r.index as usize].source() == crate::spec::passive::RuleSource::Ability)
        }
        SlotPred::HasAbilityNamed(n) => {
            let Some(src) = g.g().st.slot_pokemon(s.p as usize, s.s) else { return Ok(false) };
            g.powers(src)?.iter().any(|r| {
                let pw = &g.g().st.cdef(r.card).powers[r.index as usize];
                pw.source() == crate::spec::passive::RuleSource::Ability && pw.name == *n
            })
        }
        _ => slot_pred(g.g(), me, s, sp).expect("pure slot predicate"),
    })
}

/// The selected Pokémon, with checked reads in the filters.
pub fn slots_m<F: crate::derived::Facts>(g: &mut F, me: CardId, f: &Frame, sel: &SlotSel) -> R<SVec<SlotRef, { crate::state::MAX_SLOT_REFS }>> {
    match sel {
        SlotSel::Filtered(inner, sp) => {
            let mut out: SVec<SlotRef, { crate::state::MAX_SLOT_REFS }> = SVec::new();
            for s in slots_m(g, me, f, inner)?.iter() {
                if slot_pred_m(g, me, *s, sp)? {
                    out.push(*s);
                }
            }
            Ok(out)
        }
        SlotSel::Cancelable(inner) => slots_m(g, me, f, inner),
        _ => Ok(slots_of(g.g(), me, f, sel)),
    }
}

/// `num` with checked reads (CheckProvidedEnergy, CheckPokemonType).
pub fn num_m<F: crate::derived::Facts>(g: &mut F, me: CardId, f: &Frame, n: &Num) -> R<i32> {
    Ok(match n {
        Num::Add(a, b) => num_m(g, me, f, a)? + num_m(g, me, f, b)?,
        Num::Sub(a, b) => num_m(g, me, f, a)? - num_m(g, me, f, b)?,
        Num::Mul(a, b) => num_m(g, me, f, a)? * num_m(g, me, f, b)?,
        Num::Min(a, b) => num_m(g, me, f, a)?.min(num_m(g, me, f, b)?),
        Num::Max(a, b) => num_m(g, me, f, a)?.max(num_m(g, me, f, b)?),
        Num::If(c, a, b) => {
            if cond_m(g, me, f, c)? {
                num_m(g, me, f, a)?
            } else {
                num_m(g, me, f, b)?
            }
        }
        Num::SlotCount(sel, sp) => {
            let mut n = 0;
            for s in slots_m(g, me, f, sel)?.iter() {
                if slot_pred_m(g, me, *s, sp)? {
                    n += 1;
                }
            }
            n
        }
        Num::EnergyOn(sel, unit) => {
            let mut total = 0;
            for s in slots_m(g, me, f, sel)?.iter() {
                total += match unit {
                    EnergyUnit::SpecialEnergyCards => g
                        .g()
                        .st
                        .slot(s.p as usize, s.s)
                        .cards()
                        .iter()
                        .filter(|c| {
                            let d = g.g().st.cdef(*c);
                            d.is_energy() && d.energy_type == EnergyType::Special as u8
                        })
                        .count() as i32,
                    EnergyUnit::Provided(_) | EnergyUnit::ProvidedUnits | EnergyUnit::MatchingEntries(_) => {
                        let energy_map = g.provided_energy(s.p as usize, *s)?;
                        let mut k = 0;
                        for em in energy_map.iter() {
                            k += match unit {
                                EnergyUnit::Provided(t) => em.provides.iter().filter(|x| **x == *t || **x == ct::ANY).count() as i32,
                                EnergyUnit::MatchingEntries(t) => em.provides.iter().any(|x| *x == *t || *x == ct::ANY) as i32,
                                _ => em.provides.len() as i32,
                            };
                        }
                        k
                    }
                };
            }
            total
        }
        Num::RetreatCostColorless(w) => {
            let o = f.who(*w);
            g.retreat_cost(o)?.iter().filter(|t| **t == ct::COLORLESS).count() as i32
        }
        Num::CostNow => {
            let Some((p, _, attack, _)) = crate::prefabs::attack_data(g.g(), f.eff) else { return Ok(0) };
            g.attack_cost(p as usize, attack)?.len() as i32
        }
        _ => num(g.g(), me, f, n),
    })
}

/// Does the answer of the condition stay the same through an attack's damage? Only these can be
/// decided at step D for a choice nested under an `If` (the attack-choice rule): the player's own
/// hand, discard pile and deck, their own Bench, printed properties of the Active Pokémon and of
/// this Pokémon, and the cards a step D choice has already picked. Anything else (damage, Special
/// Conditions, the opponent's cards, a coin) is decided when it is reached.
/// Does the condition read which cards a zone hidden from the player holds (the kinds of the cards of a deck or of the
/// opponent's hand: "if your deck has a card that evolves from this Pokémon", the copies of a named card still unseen)?
/// How many cards it holds is public; what they are isn't.
pub fn cond_reads_hidden_cards(c: &Cond) -> bool {
    fn hidden(z: &ZoneRef, p: &Pred) -> bool {
        (z.1 == Zone::Deck || (z.0 == Who::Opp && z.1 == Zone::Hand)) && !matches!(p, Pred::Any)
    }
    fn num_reads(n: &Num) -> bool {
        match n {
            Num::CardCount(z, p) | Num::DistinctTypes(z, p) | Num::OthersCount(z, p) => hidden(z, p),
            Num::KnownCopies(..) => true,
            Num::Add(a, b) | Num::Sub(a, b) | Num::Mul(a, b) | Num::Min(a, b) | Num::Max(a, b) => num_reads(a) || num_reads(b),
            Num::If(c, a, b) => cond_reads_hidden_cards(c) || num_reads(a) || num_reads(b),
            _ => false,
        }
    }
    match c {
        Cond::Printed(c) | Cond::Not(c) => cond_reads_hidden_cards(c),
        Cond::All(cs) | Cond::Any(cs) => cs.iter().any(cond_reads_hidden_cards),
        Cond::Cmp(a, _, b) => num_reads(a) || num_reads(b),
        Cond::Nonempty(z, p) | Cond::NonemptyOther(z, p) => hidden(z, p),
        // Whether a card that evolves from a Pokémon in play exists stands for whether the deck could hold one.
        Cond::KnownCopies { .. } | Cond::AllNamesKnown { .. } | Cond::HasEvolutionInPool(_) => true,
        _ => false,
    }
}

pub fn cond_stable(c: &Cond) -> bool {
    fn own_zone(z: &ZoneRef) -> bool {
        z.0 == Who::Me && matches!(z.1, Zone::Hand | Zone::Discard | Zone::Deck)
    }
    fn slot_pred(p: &SlotPred) -> bool {
        match p {
            SlotPred::Any | SlotPred::IsActive | SlotPred::IsBench | SlotPred::Basic | SlotPred::Top(_) | SlotPred::Tag(_) | SlotPred::HasCard(_) => true,
            SlotPred::Not(p) => slot_pred(p),
            SlotPred::All(ps) | SlotPred::OneOf(ps) => ps.iter().all(slot_pred),
            _ => false,
        }
    }
    fn slot_sel(s: &SlotSel) -> bool {
        match s {
            SlotSel::One(SlotExpr::This | SlotExpr::Active(_)) => true,
            SlotSel::Bench(Who::Me) => true,
            _ => false,
        }
    }
    fn num_stable(n: &Num) -> bool {
        match n {
            Num::Lit(_) => true,
            Num::ZoneSize(z) | Num::CardCount(z, _) => own_zone(z),
            _ => false,
        }
    }
    match c {
        Cond::True | Cond::False | Cond::Chosen(_) => true,
        Cond::Not(c) => cond_stable(c),
        Cond::All(cs) | Cond::Any(cs) => cs.iter().all(cond_stable),
        Cond::Cmp(a, _, b) => num_stable(a) && num_stable(b),
        Cond::Nonempty(z, _) | Cond::NonemptyOther(z, _) | Cond::ZoneIs(z, _) => own_zone(z),
        Cond::Slot(SlotExpr::This | SlotExpr::Active(_), p) => slot_pred(p),
        Cond::AnySlot(s, p) | Cond::AllSlots(s, p) => slot_sel(s) && slot_pred(p),
        Cond::StadiumInPlay(_) => true,
        _ => false,
    }
}

/// `cond` with checked reads.
pub fn cond_m<F: crate::derived::Facts>(g: &mut F, me: CardId, f: &Frame, c: &Cond) -> R<bool> {
    Ok(match c {
        Cond::Not(c) => !cond_m(g, me, f, c)?,
        Cond::Printed(c) => cond_m(g, me, f, c)?,
        Cond::All(cs) => {
            for c in cs.iter() {
                if !cond_m(g, me, f, c)? {
                    return Ok(false);
                }
            }
            true
        }
        Cond::Any(cs) => {
            for c in cs.iter() {
                if cond_m(g, me, f, c)? {
                    return Ok(true);
                }
            }
            false
        }
        Cond::Cmp(a, op, b) => {
            let (a, b) = (num_m(g, me, f, a)?, num_m(g, me, f, b)?);
            match op {
                CmpOp::Lt => a < b,
                CmpOp::Le => a <= b,
                CmpOp::Eq => a == b,
                CmpOp::Ne => a != b,
                CmpOp::Ge => a >= b,
                CmpOp::Gt => a > b,
            }
        }
        Cond::CanEvolveSkippingStage1 => super::ops::board::can_evolve_skipping_stage1(g, me, f.p as usize)?,
        Cond::AbilityBlocked => g.has_no_ability(me, None),
        Cond::CanEvolveBasic(w) => super::ops::board::evolve_targets(g, f.who(*w), f.cause)?.0,
        Cond::ToolBlocked => g.tools_off(),
        Cond::Slot(e, sp) => match slot_of(g.g(), me, f, *e) {
            Some(s) => slot_pred_m(g, me, s, sp)?,
            None => false,
        },
        Cond::TypesShared(a, b) => {
            // The types of the Pokémon in play after effects (`derived::types`), never the printed types.
            let mut mine: SVec<CardType, 16> = SVec::new();
            for s in slots_m(g, me, f, a)?.iter() {
                for t in g.types(*s)?.iter() {
                    if !mine.contains(t) {
                        mine.push(*t);
                    }
                }
            }
            for s in slots_m(g, me, f, b)?.iter() {
                if g.types(*s)?.iter().any(|t| mine.contains(t)) {
                    return Ok(true);
                }
            }
            false
        }
        Cond::AnySlot(sel, sp) => {
            for s in slots_m(g, me, f, sel)?.iter() {
                if slot_pred_m(g, me, *s, sp)? {
                    return Ok(true);
                }
            }
            false
        }
        Cond::AllSlots(sel, sp) => {
            let v = slots_m(g, me, f, sel)?;
            if v.is_empty() {
                return Ok(false);
            }
            for s in v.iter() {
                if !slot_pred_m(g, me, *s, sp)? {
                    return Ok(false);
                }
            }
            true
        }
        _ => cond(g.g(), me, f, c),
    })
}


/// Does the number need a checked read (`num_m`)?
pub fn num_is_checked(n: &Num) -> bool {
    match n {
        Num::SlotCount(..) | Num::EnergyOn(..) | Num::CostNow | Num::RetreatCostColorless(_) => true,
        Num::Add(a, b) | Num::Sub(a, b) | Num::Mul(a, b) | Num::Min(a, b) | Num::Max(a, b) => num_is_checked(a) || num_is_checked(b),
        Num::If(_, a, b) => num_is_checked(a) || num_is_checked(b),
        _ => false,
    }
}

#[cfg(test)]
mod phrase_tests {
    //! "Pokémon ex", "Pokémon V", the Rule Box of the top card (APR D-07; id1667; Neutralization Zone: "(Pokémon ex, Pokémon V,
    //! etc. have Rule Boxes.)").
    use super::*;

    const NAMES: [&str; 3] = ["Duskull PRE 35", "Meowth ex POR 62", "Slowpoke MEP 86"];

    fn game() -> Game {
        let deck: Vec<u16> = NAMES.iter().flat_map(|n| std::iter::repeat(*n).take(4)).chain(std::iter::repeat("Psychic Energy MEE 5").take(48)).map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g
    }

    fn card(g: &Game, name: &str, p: usize) -> CardId {
        let def = crate::carddb::def_by_full_name(name).unwrap();
        g.st.cards.iter().position(|c| c.def == def && c.owner as usize == p).unwrap() as CardId
    }

    /// A Pokémon is its top card: a Rule Box on a card under it isn't the Pokémon's (no pool line evolves from an ex into a
    /// non-ex, so this is a constructed stack).
    #[test]
    fn rule_box_is_the_top_cards() {
        let mut g = game();
        let (ex, plain) = (card(&g, "Meowth ex POR 62", 0), card(&g, "Duskull PRE 35", 0));
        let active = g.st.players[0].active;
        let s = SlotRef::new(0, active);
        g.st.slot_edit(0, active).cards().clear();
        g.st.slot_edit(0, active).cards().push(ex);
        g.st.slot_edit(0, active).cards().push(plain);
        assert_eq!(slot_pred(&g, plain, s, &SlotPred::RuleBox), Some(false));
        assert_eq!(slot_pred(&g, plain, s, &SlotPred::Top(Pred::PokemonEx)), Some(false));
        g.st.slot_edit(0, active).cards().clear();
        g.st.slot_edit(0, active).cards().push(plain);
        g.st.slot_edit(0, active).cards().push(ex);
        assert_eq!(slot_pred(&g, ex, s, &SlotPred::RuleBox), Some(true));
        assert_eq!(slot_pred(&g, ex, s, &SlotPred::Top(Pred::PokemonEx)), Some(true));
        assert_eq!(slot_pred(&g, ex, s, &SlotPred::Top(Pred::PokemonV)), Some(false));
    }

    /// The card predicates over cards: an ex is "Pokémon ex" and has a Rule Box, a plain Pokémon is neither, no pool card is a
    /// "Pokémon V".
    #[test]
    fn phrase_predicates_over_cards() {
        let g = game();
        let (ex, plain) = (card(&g, "Meowth ex POR 62", 0), card(&g, "Duskull PRE 35", 0));
        assert!(pred(&g, ex, &Pred::PokemonEx) && pred(&g, ex, &Pred::RuleBox) && !pred(&g, ex, &Pred::PokemonV));
        assert!(!pred(&g, plain, &Pred::PokemonEx) && !pred(&g, plain, &Pred::RuleBox) && !pred(&g, plain, &Pred::PokemonV));
        assert!(pred(&g, ex, &Pred::OneOf(&[Pred::PokemonEx, Pred::PokemonV])));
        assert!(!crate::carddb::cards().iter().any(|d| d.is_pokemon_v()));
    }
}
