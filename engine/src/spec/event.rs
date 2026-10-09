//! Rules events as data, and the one predicate type over them (docs/design/events-design.md, sections 4
//! and 5): triggers, locks, prevention, permissions and restrictions will all be `EventPred`s.
//!
//! Events batch 1: the types and their evaluation only. Nothing in the engine builds an `EventView` or
//! reads an `EventPred` yet; batch 2 (EnterPlay, Evolve) is the first user. The trees are static data
//! (`&'static` slices, like `Pred` and `SlotPred`); evaluating one allocates nothing.

use super::trigger::Turn;
use super::value::{pred, slot_pred, Pred, SlotPred, Who, Zone};
use crate::cause::{Cause, CauseKind};
use crate::effects::SlotRef;
use crate::game::Game;
use crate::list::CardId;
use crate::types::SpecialCondition;

/// Every rules event of the design's table (section 4), one per physical action at the table, plus the
/// turn structure and the turn actions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EventKind {
    /// A Pokémon card goes onto an empty spot (played from the hand by the rule, put by an effect, setup).
    EnterPlay,
    Evolve,
    Devolve,
    /// The Pokémon card in a spot is replaced by another.
    Swap,
    /// An Energy or a Tool is attached.
    Attach,
    MoveEnergy,
    MoveTool,
    PlayTrainer,
    /// The Active Pokémon changes (retreat, switch, switch-in, switch-out, promotion, setup).
    ChangeActive,
    /// Attack damage.
    Damage,
    PlaceCounters,
    MoveCounters,
    /// Healing.
    RemoveCounters,
    GainCondition,
    RemoveCondition,
    KnockOut,
    TakePrizes,
    Discard,
    Draw,
    PutIntoHand,
    PutIntoDeck,
    LeavePlay,
    Shuffle,
    Look,
    Reveal,
    CoinFlip,
    StateCheck,
    GameEnd,
    Mulligan,
    SetPrizes,
    BeginTurn,
    EndTurn,
    Checkup,
    UseAttack,
    UseAbility,
    UseStadium,
    Retreat,
}

/// How a Pokémon evolved.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EvolvePath {
    /// Evolving from the hand by the game rule, or by Rare Candy.
    Rule,
    /// Put onto the Pokémon by an effect.
    Effect,
}

/// A predicate over a `Cause`. "Me" / "Opp" are relative to the owner of the card that declares it.
pub enum CausePred {
    /// The cause is of this kind (`Rule { which }` matches that rule only).
    Kind(CauseKind),
    /// Any game rule.
    Rule,
    /// Caused by this card's owner (`Me`) or by the opponent (`Opp`).
    By(Who),
    /// The causing card matches.
    Card(Pred),
    All(&'static [CausePred]),
    Any(&'static [CausePred]),
    Not(&'static CausePred),
}

/// The one predicate over events. "Me" / "Opp" are relative to the owner of the card that declares it.
pub enum EventPred {
    Kind(EventKind),
    /// Done by the game rule as the player's own action: a Pokémon played from the hand (EnterPlay mode
    /// `rule`), the turn's one Energy attachment, a Tool played from the hand.
    Manual(bool),
    /// The zone the card comes from.
    Source(Zone),
    Path(EvolvePath),
    Cause(CausePred),
    /// The event's card matches.
    Card(Pred),
    /// The event's card is the declaring card (the design's `Card(This)`).
    This,
    /// The Pokémon evolved from (Evolve) matches.
    Base(Pred),
    /// The spot the event is about matches.
    Slot(SlotPred),
    /// Whose card / Pokémon the event is about.
    Owner(Who),
    /// Whose turn it is, relative to the event's owner (`Owner`: the event's owner's turn).
    Turn(Turn),
    /// The Special Condition gained or removed (not in the design's list; its Slowpoke example needs it).
    Condition(SpecialCondition),
    All(&'static [EventPred]),
    Any(&'static [EventPred]),
    Not(&'static EventPred),
}

/// The attributes of one event that predicates read.
#[derive(Clone, Copy, Debug)]
pub struct EventView {
    pub kind: EventKind,
    pub source: Option<Zone>,
    pub manual: bool,
    pub path: Option<EvolvePath>,
    pub cause: Cause,
    pub card: Option<CardId>,
    pub base: Option<CardId>,
    pub slot: Option<SlotRef>,
    pub condition: Option<SpecialCondition>,
    /// The player whose card / Pokémon the event is about.
    pub owner: u8,
    /// The player whose turn it is.
    pub turn: u8,
}

impl EventView {
    /// An event of `kind` about `owner`'s card, with nothing else set.
    pub const fn new(kind: EventKind, cause: Cause, owner: u8, turn: u8) -> EventView {
        EventView { kind, source: None, manual: false, path: None, cause, card: None, base: None, slot: None, condition: None, owner, turn }
    }
}

fn who_is(g: &Game, me: CardId, w: Who, player: u8) -> bool {
    let mine = g.st.owner(me) as u8 == player;
    match w {
        Who::Me => mine,
        Who::Opp => !mine,
    }
}

impl CausePred {
    pub const fn attack() -> CausePred {
        CausePred::Kind(CauseKind::Attack)
    }
    pub const fn ability() -> CausePred {
        CausePred::Kind(CauseKind::Ability)
    }
    pub const fn opponents() -> CausePred {
        CausePred::By(Who::Opp)
    }

    /// Does the cause match, for the card `me` that declares the predicate?
    pub fn eval(&self, g: &Game, me: CardId, c: &Cause) -> bool {
        match self {
            CausePred::Kind(k) => c.kind == *k,
            CausePred::Rule => matches!(c.kind, CauseKind::Rule { .. }),
            CausePred::By(w) => who_is(g, me, *w, c.player),
            CausePred::Card(p) => c.card.map_or(false, |x| pred(g, x, p)),
            CausePred::All(ps) => ps.iter().all(|p| p.eval(g, me, c)),
            CausePred::Any(ps) => ps.iter().any(|p| p.eval(g, me, c)),
            CausePred::Not(p) => !p.eval(g, me, c),
        }
    }
}

impl EventPred {
    pub const fn kind(k: EventKind) -> EventPred {
        EventPred::Kind(k)
    }
    pub const fn from_hand() -> EventPred {
        EventPred::Source(Zone::Hand)
    }
    pub const fn cause(c: CausePred) -> EventPred {
        EventPred::Cause(c)
    }

    /// Does the event match, for the card `me` that declares the predicate? A slot predicate that needs a
    /// checked read (`SlotPred::TypeIs`, `HasAbility`, ...) answers false until the derived layer exists.
    pub fn eval(&self, g: &Game, me: CardId, v: &EventView) -> bool {
        match self {
            EventPred::Kind(k) => v.kind == *k,
            EventPred::Manual(m) => v.manual == *m,
            EventPred::Source(z) => v.source == Some(*z),
            EventPred::Path(p) => v.path == Some(*p),
            EventPred::Cause(c) => c.eval(g, me, &v.cause),
            EventPred::Card(p) => v.card.map_or(false, |c| pred(g, c, p)),
            EventPred::This => v.card == Some(me),
            EventPred::Base(p) => v.base.map_or(false, |c| pred(g, c, p)),
            EventPred::Slot(sp) => v.slot.map_or(false, |s| slot_pred(g, me, s, sp).unwrap_or(false)),
            EventPred::Owner(w) => who_is(g, me, *w, v.owner),
            EventPred::Turn(t) => match t {
                Turn::Owner => v.turn == v.owner,
                Turn::Opp => v.turn != v.owner,
                Turn::Any => true,
            },
            EventPred::Condition(c) => v.condition == Some(*c),
            EventPred::All(ps) => ps.iter().all(|p| p.eval(g, me, v)),
            EventPred::Any(ps) => ps.iter().any(|p| p.eval(g, me, v)),
            EventPred::Not(p) => !p.eval(g, me, v),
        }
    }
}

#[cfg(test)]
mod tests {
    //! The design's examples (section 5) against hand-built events.
    use super::*;
    use crate::cause::RuleWhich;
    use crate::state::AttackRef;
    use crate::types::{ct, tag};

    const NAMES: [&str; 7] = ["Duskull PRE 35", "Team Rocket's Murkrow DRI 127", "Meowth ex POR 62", "Slowpoke MEP 86", "Team Rocket's Mewtwo ex DRI 81", "Risky Ruins MEG 127", "Rare Candy MEG 125"];

    fn game() -> Game {
        let deck: Vec<u16> = NAMES.iter().flat_map(|n| std::iter::repeat(*n).take(4)).chain(std::iter::repeat("Psychic Energy MEE 5").take(32)).map(|n| crate::carddb::def_by_full_name(n).unwrap()).collect();
        let mut g = Game::new(7);
        g.start([&deck, &deck]).unwrap();
        g
    }

    /// A copy of `name` owned by `p`.
    fn card(g: &Game, name: &str, p: usize) -> CardId {
        let def = crate::carddb::def_by_full_name(name).unwrap();
        g.st.cards.iter().position(|c| c.def == def && c.owner as usize == p).unwrap() as CardId
    }

    fn bench_slot(g: &Game, p: usize) -> SlotRef {
        let pl = &g.st.players[p];
        let s = (0..pl.slots.len() as u8).find(|s| *s != pl.active).unwrap();
        SlotRef::new(p, s)
    }

    fn enter(g: &Game, c: CardId, from: Zone, cause: Cause, turn: u8) -> EventView {
        let p = g.st.owner(c);
        EventView { source: Some(from), card: Some(c), slot: Some(bench_slot(g, p)), manual: matches!(cause.kind, CauseKind::Rule { which: RuleWhich::Action }), ..EventView::new(EventKind::EnterPlay, cause, p as u8, turn) }
    }

    /// Risky Ruins: a Basic non-[D] Pokémon put onto its owner's Bench during their turn, any source
    /// (id2233). Retreating isn't EnterPlay (id2329).
    static RISKY_RUINS: EventPred = EventPred::All(&[
        EventPred::Kind(EventKind::EnterPlay),
        EventPred::Slot(SlotPred::IsBench),
        EventPred::Card(Pred::All(&[Pred::Basic, Pred::Not(&Pred::PokemonType(ct::DARK))])),
        EventPred::Turn(Turn::Owner),
    ]);

    #[test]
    fn risky_ruins() {
        let g = game();
        let ruins = card(&g, "Risky Ruins MEG 127", 0);
        let duskull = card(&g, "Duskull PRE 35", 1);
        let murkrow = card(&g, "Team Rocket's Murkrow DRI 127", 1);
        assert!(pred(&g, murkrow, &Pred::PokemonType(ct::DARK)));
        let own_ability = Cause::new(CauseKind::Ability, Some(duskull), 1);
        // Duskull's Come and Get You puts it from the discard pile: Risky Ruins applies (the bug today).
        assert!(RISKY_RUINS.eval(&g, ruins, &enter(&g, duskull, Zone::Discard, own_ability, 1)));
        assert!(RISKY_RUINS.eval(&g, ruins, &enter(&g, duskull, Zone::Hand, Cause::rule(RuleWhich::Action, 1), 1)));
        // Not during the opponent's turn, not a [D] Pokémon, not a retreat.
        assert!(!RISKY_RUINS.eval(&g, ruins, &enter(&g, duskull, Zone::Hand, Cause::rule(RuleWhich::Action, 1), 0)));
        assert!(!RISKY_RUINS.eval(&g, ruins, &enter(&g, murkrow, Zone::Hand, Cause::rule(RuleWhich::Action, 1), 1)));
        let retreat = EventView { kind: EventKind::ChangeActive, ..enter(&g, duskull, Zone::Hand, Cause::rule(RuleWhich::Retreat, 1), 1) };
        assert!(!RISKY_RUINS.eval(&g, ruins, &retreat));
    }

    /// Meowth ex Last-Ditch Catch: played from the hand onto the Bench.
    static MEOWTH: EventPred = EventPred::All(&[EventPred::Kind(EventKind::EnterPlay), EventPred::This, EventPred::Source(Zone::Hand), EventPred::Slot(SlotPred::IsBench)]);

    #[test]
    fn meowth_ex() {
        let g = game();
        let meowth = card(&g, "Meowth ex POR 62", 0);
        let other = card(&g, "Meowth ex POR 62", 1);
        let rule = Cause::rule(RuleWhich::Action, 0);
        assert!(MEOWTH.eval(&g, meowth, &enter(&g, meowth, Zone::Hand, rule, 0)));
        assert!(!MEOWTH.eval(&g, meowth, &enter(&g, meowth, Zone::Deck, rule, 0)));
        assert!(!MEOWTH.eval(&g, meowth, &enter(&g, other, Zone::Hand, rule, 0)));
    }

    /// Team Rocket's Arbok: no Pokémon with an Ability (except Team Rocket's) played from the hand or
    /// evolved from the hand. Rare Candy is covered; Grand Tree isn't (id1133).
    static ARBOK: EventPred = EventPred::All(&[
        EventPred::Any(&[EventPred::Kind(EventKind::EnterPlay), EventPred::Kind(EventKind::Evolve)]),
        EventPred::Source(Zone::Hand),
        EventPred::Card(Pred::All(&[Pred::PrintsAbility, Pred::Not(&Pred::Tag(tag::TEAM_ROCKET))])),
    ]);

    #[test]
    fn team_rockets_arbok() {
        let g = game();
        let arbok = card(&g, "Duskull PRE 35", 0); // the declaring card's owner is all that matters here
        let slowpoke = card(&g, "Slowpoke MEP 86", 1);
        let mewtwo = card(&g, "Team Rocket's Mewtwo ex DRI 81", 1);
        let candy = card(&g, "Rare Candy MEG 125", 1);
        assert!(pred(&g, mewtwo, &Pred::PrintsAbility) && pred(&g, mewtwo, &Pred::Tag(tag::TEAM_ROCKET)));
        let rule = Cause::rule(RuleWhich::Action, 1);
        assert!(ARBOK.eval(&g, arbok, &enter(&g, slowpoke, Zone::Hand, rule, 1)));
        assert!(!ARBOK.eval(&g, arbok, &enter(&g, mewtwo, Zone::Hand, rule, 1)));
        let evolve = |from: Zone, path: EvolvePath, cause: Cause| EventView { kind: EventKind::Evolve, path: Some(path), ..enter(&g, slowpoke, from, cause, 1) };
        // Rare Candy: from the hand, by a Trainer.
        assert!(ARBOK.eval(&g, arbok, &evolve(Zone::Hand, EvolvePath::Rule, Cause::new(CauseKind::Trainer, Some(candy), 1))));
        // Grand Tree: from the deck, by a Stadium.
        assert!(!ARBOK.eval(&g, arbok, &evolve(Zone::Deck, EvolvePath::Effect, Cause::new(CauseKind::Stadium, None, 1))));
    }

    /// Slowpoke Dopey Face: can't be Confused, whatever causes it (Lisia's Appeal included).
    static DOPEY_FACE: EventPred = EventPred::All(&[EventPred::Kind(EventKind::GainCondition), EventPred::Condition(SpecialCondition::Confused)]);
    /// Hide 'n' Sneak: prevent the effects of the opponent's attacks and Abilities.
    static HIDE_N_SNEAK: EventPred = EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Any(&[CausePred::Kind(CauseKind::Attack), CausePred::Kind(CauseKind::Ability)])]));

    #[test]
    fn slowpoke_and_hide_n_sneak() {
        let g = game();
        let slowpoke = card(&g, "Slowpoke MEP 86", 0);
        let lisia = Cause::new(CauseKind::Trainer, None, 1);
        let confuse = |cause: Cause, c: SpecialCondition| EventView { slot: Some(bench_slot(&g, 0)), condition: Some(c), ..EventView::new(EventKind::GainCondition, cause, 0, 1) };
        assert!(DOPEY_FACE.eval(&g, slowpoke, &confuse(lisia, SpecialCondition::Confused)));
        assert!(!DOPEY_FACE.eval(&g, slowpoke, &confuse(lisia, SpecialCondition::Poisoned)));
        let attack = Cause::attack(1, None, AttackRef { card: 0, index: 0 });
        assert!(HIDE_N_SNEAK.eval(&g, slowpoke, &confuse(attack, SpecialCondition::Confused)));
        assert!(HIDE_N_SNEAK.eval(&g, slowpoke, &confuse(Cause::new(CauseKind::Ability, None, 1), SpecialCondition::Confused)));
        assert!(!HIDE_N_SNEAK.eval(&g, slowpoke, &confuse(lisia, SpecialCondition::Confused)), "a Trainer isn't an attack or an Ability");
        assert!(!HIDE_N_SNEAK.eval(&g, slowpoke, &confuse(Cause::attack(0, None, AttackRef { card: 0, index: 0 }), SpecialCondition::Confused)), "its own side's attack");
    }
}
