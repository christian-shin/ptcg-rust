//! Rules events as data, and the one predicate type over them (docs/design/events-design.md, sections 4
//! and 5): triggers, locks, prevention, permissions and restrictions are all `EventPred`s.
//!
//! Events batch 2: EnterPlay, Evolve, Devolve and Swap are built by their routines (`engine/enter.rs`); events
//! batch 3: Attach, MoveEnergy and MoveTool by theirs (`engine/attach.rs`); events batch 4: GainCondition,
//! RemoveCondition, RemoveCounters (healing) and CoinFlip by theirs (`engine/condition.rs`). They are read by triggers (`trigger::Event::On`), locks (`LockDecl::forbids`), permissions (`Modifier::Permit`)
//! and restrictions (`CardSpec::restricts`). The trees are static data (`&'static` slices, like `Pred` and
//! `SlotPred`); evaluating one allocates nothing.

use super::value::{pred, slot_pred, slot_pred_m, Pred, SlotPred, Who};
use crate::cause::{Cause, CauseKind};
use crate::effects::{KindMask, SlotRef};
use crate::game::{Game, R};
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

impl EventKind {
    /// The effect kind that carries the event (`effects::k`), for the events that have one so far.
    pub const fn effect_kind(self) -> Option<u32> {
        use crate::effects::k;
        match self {
            EventKind::EnterPlay => Some(k::ENTER_PLAY),
            EventKind::Evolve => Some(k::EVOLVE),
            EventKind::Devolve => Some(k::DEVOLVE),
            EventKind::Swap => Some(k::SWAP),
            EventKind::Attach => Some(k::ATTACH),
            EventKind::MoveEnergy => Some(k::MOVE_ENERGY),
            EventKind::MoveTool => Some(k::MOVE_TOOL),
            EventKind::GainCondition => Some(k::GAIN_CONDITION),
            EventKind::RemoveCondition => Some(k::REMOVE_CONDITION),
            EventKind::RemoveCounters => Some(k::HEAL),
            EventKind::CoinFlip => Some(k::COIN_FLIP),
            _ => None,
        }
    }
}

/// What a coin is flipped for (the CoinFlip event).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoinPurpose {
    /// A card's text ("flip a coin"): an attack, an Ability, a Trainer, a Tool, a lasting effect (Seismitoad's
    /// Quaking Fist when a Trainer is played).
    Effect,
    /// A Confused Pokémon tries to attack (APR: Special Conditions).
    Confusion,
    /// Burned, at Pokémon Checkup.
    Burned,
    /// Asleep, at Pokémon Checkup.
    Asleep,
    /// Who goes first (setup, and the Sudden Death game).
    FirstPlayer,
}

/// How a Pokémon evolved.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EvolvePath {
    /// Evolving from the hand by the game rule, or by Rare Candy (which counts as playing it from the hand,
    /// id285, id1998): the rule's limits apply (APR A-05).
    Rule,
    /// Put onto the Pokémon by an effect ("put it onto that Pokémon to evolve it"): the rule's limits don't
    /// apply unless the card restricts it (APR C-12).
    Effect,
}

/// How a Pokémon card entered play (EnterPlay).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnterMode {
    /// Played from the hand by the game rule: a Basic Pokémon onto the Bench, a Fossil played as a Pokémon
    /// (whose cause is the Trainer card: the mode is never derived from the cause).
    Rule,
    /// Put into play by an effect, from any zone (the hand included).
    Effect,
    /// Put down while setting up the game.
    Setup,
}

/// The rules zone a card comes from: never a staging list. A card looked at or searched for is in the zone
/// its search started from (`enter::rules_zone`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RulesZone {
    Hand,
    Deck,
    Discard,
    Prizes,
    LostZone,
    /// A Pokémon in play (the cards of a slot).
    InPlay,
    Stadium,
}

/// A limit on evolving that a `Permit` lifts or a `Restrict` imposes (APR A-05).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Limit {
    /// Not during the owner's first turn.
    FirstTurn,
    /// Not a Pokémon that came into play this turn: put into play, evolved or devolved this turn (id2229: a
    /// Pokémon evolved this turn "has come into play that turn", so this is also the once-per-turn limit;
    /// APR C-13 for devolving).
    BaseEnteredThisTurn,
    /// The Evolution card must evolve from the Pokémon ("Evolves from" names it). Only a `Permit` lifts it
    /// (Rainbow DNA); a `Restrict` doesn't impose it.
    EvolvesFrom,
}

/// The role the declaring card plays in an event ("this Pokémon", "this card").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    /// The event's card (the Pokémon entering play, the Evolution card): "when you play this Pokémon".
    Card,
    /// The Pokémon evolved from (Evolve) or devolved (Devolve): "this Pokémon can evolve".
    Base,
    /// The card whose attack, Ability, Trainer, Stadium, Tool or Energy caused the event: "you can't use
    /// this card on ...".
    CauseCard,
}

/// Whose turn it is, for an event predicate (separate from the trigger's `trigger::Turn`, whose referent is
/// the declaring card's owner).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TurnOf {
    /// The turn of the player whose card / Pokémon the event is about ("during their turn").
    EventOwner,
    /// Not that player's turn.
    NotEventOwner,
    /// The turn of the declaring card's owner ("during your turn").
    Me,
    /// The turn of the declaring card owner's opponent.
    Opp,
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
    /// The turn's one Energy attachment by the game rule (Attach, events batch 3). Nothing else is manual.
    Manual(bool),
    /// How the Pokémon entered play (EnterPlay).
    Mode(EnterMode),
    /// The rules zone the card comes from.
    Source(RulesZone),
    Path(EvolvePath),
    Cause(CausePred),
    /// The event's card matches (as printed; a Fossil in play is a Pokémon).
    Card(Pred),
    /// The Pokémon evolved from (Evolve), or the Pokémon devolved (Devolve), matches (its top card, as
    /// printed).
    Base(Pred),
    /// The declaring card has this role in the event: `This(Role::Card)` "when you play this Pokémon",
    /// `This(Role::Base)` "this Pokémon can evolve", `This(Role::CauseCard)` "you can't use this card".
    This(Role),
    /// The spot the event is about matches (with a checked read when the predicate needs one).
    Slot(SlotPred),
    /// Whose card / Pokémon the event is about.
    Owner(Who),
    /// Whose turn it is.
    Turn(TurnOf),
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
    pub source: Option<RulesZone>,
    /// EnterPlay: how it entered.
    pub mode: Option<EnterMode>,
    /// Attach: the turn's one Energy attachment by the game rule (events batch 3); false for every other event.
    pub manual: bool,
    pub path: Option<EvolvePath>,
    pub cause: Cause,
    pub card: Option<CardId>,
    pub base: Option<CardId>,
    pub slot: Option<SlotRef>,
    pub condition: Option<SpecialCondition>,
    /// RemoveCounters: the HP of damage counters removed.
    pub amount: i32,
    /// CoinFlip: what it is for and the result.
    pub purpose: Option<CoinPurpose>,
    pub heads: Option<bool>,
    /// The player whose card / Pokémon the event is about.
    pub owner: u8,
    /// The player whose turn it is.
    pub turn: u8,
    /// Evolve: the Pokémon evolved from came into play this turn (the slot's entered-turn record,
    /// `Slot::entered_turn`, which no permission rewrites).
    pub base_entered_this_turn: bool,
    /// Evolve: it is the owner's first turn (the game's turns 1 and 2 are each player's first turn).
    pub owner_first_turn: bool,
}

impl EventView {
    /// An event of `kind` about `owner`'s card, with nothing else set.
    pub const fn new(kind: EventKind, cause: Cause, owner: u8, turn: u8) -> EventView {
        EventView { kind, source: None, mode: None, manual: false, path: None, cause, card: None, base: None, slot: None, condition: None, amount: 0, purpose: None, heads: None, owner, turn, base_entered_this_turn: false, owner_first_turn: false }
    }

    /// The player doing the action: the `Cause` player (who plays the card, uses the Ability, attack or
    /// Trainer that does it; the player the rule acts for). A lock binds the actor, not the owner of the
    /// card: "your opponent can't play X from their hand" stops the opponent's own plays, by the rule or by
    /// their Abilities and attacks (id25, id230), and not a card the lock's owner puts from the opponent's
    /// hand (id959: putting the opponent's Basic onto their Bench isn't them playing it; Mandibuzz's Look for
    /// Prey past Team Rocket's Arbok).
    #[inline]
    pub const fn actor(&self) -> u8 {
        self.cause.player
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

/// Every effect kind that carries an event so far.
pub const EVENT_KINDS: KindMask = crate::effects::mask(&[
    crate::effects::k::ENTER_PLAY,
    crate::effects::k::EVOLVE,
    crate::effects::k::DEVOLVE,
    crate::effects::k::SWAP,
    crate::effects::k::ATTACH,
    crate::effects::k::MOVE_ENERGY,
    crate::effects::k::MOVE_TOOL,
    crate::effects::k::GAIN_CONDITION,
    crate::effects::k::REMOVE_CONDITION,
    crate::effects::k::HEAL,
    crate::effects::k::COIN_FLIP,
]);
/// The Pokémon events (events batch 2): a lock over them sets `DECLARES_EVENT_LOCK`.
pub const POKEMON_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::ENTER_PLAY, crate::effects::k::EVOLVE, crate::effects::k::DEVOLVE, crate::effects::k::SWAP]);
/// The attaching events (events batch 3): a lock over them sets `DECLARES_ATTACH_LOCK`.
pub const ATTACH_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::ATTACH, crate::effects::k::MOVE_ENERGY, crate::effects::k::MOVE_TOOL]);
/// The Special Condition events (events batch 4): a lock over them sets `DECLARES_CONDITION_LOCK`, a `Prevent`
/// `DECLARES_CONDITION_PREVENT`.
pub const CONDITION_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::GAIN_CONDITION, crate::effects::k::REMOVE_CONDITION]);
/// RemoveCounters (healing): `DECLARES_HEAL_LOCK` / `DECLARES_HEAL_PREVENT`.
pub const HEAL_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::HEAL]);
/// CoinFlip: `DECLARES_COIN_LOCK` / `DECLARES_COIN_PREVENT`.
pub const COIN_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::COIN_FLIP]);

impl EventPred {
    /// Matches no event (`LockDecl::forbids` of a lock that declares only old `LockedAction`s).
    pub const NEVER: EventPred = EventPred::Any(&[]);
    /// Matches every event.
    pub const ALWAYS: EventPred = EventPred::All(&[]);

    pub const fn kind(k: EventKind) -> EventPred {
        EventPred::Kind(k)
    }
    pub const fn from_hand() -> EventPred {
        EventPred::Source(RulesZone::Hand)
    }
    pub const fn cause(c: CausePred) -> EventPred {
        EventPred::Cause(c)
    }

    /// Is this the predicate that matches nothing (`NEVER`)?
    pub const fn is_never(&self) -> bool {
        matches!(self, EventPred::Any(ps) if ps.is_empty())
    }

    /// Does the predicate depend on what causes the event (`Cause`, or the declaring card's role, which can be the
    /// cause's card)? "Can't be Confused" doesn't; "prevent all effects of attacks used by your opponent's Pokémon"
    /// does.
    pub const fn reads_cause(&self) -> bool {
        match self {
            EventPred::Cause(_) | EventPred::This(_) => true,
            EventPred::All(ps) | EventPred::Any(ps) => {
                let mut i = 0;
                while i < ps.len() {
                    if ps[i].reads_cause() {
                        return true;
                    }
                    i += 1;
                }
                false
            }
            EventPred::Not(p) => p.reads_cause(),
            _ => false,
        }
    }

    /// The effect kinds whose events can match: the kinds the predicate names, or every event kind with an
    /// effect where it names none. The declaring card's dispatch mask.
    pub const fn effect_kinds(&self) -> KindMask {
        match self {
            EventPred::Kind(e) => match e.effect_kind() {
                Some(x) => crate::effects::mask(&[x]),
                None => KindMask::EMPTY,
            },
            // A conjunction is limited by each member that names kinds.
            EventPred::All(ps) => {
                let mut i = 0;
                let mut m = EVENT_KINDS;
                while i < ps.len() {
                    let x = ps[i].effect_kinds();
                    m = KindMask([m.0[0] & x.0[0], m.0[1] & x.0[1], m.0[2] & x.0[2], m.0[3] & x.0[3]]);
                    i += 1;
                }
                m
            }
            EventPred::Any(ps) => {
                let mut i = 0;
                let mut m = KindMask::EMPTY;
                while i < ps.len() {
                    m = m.or(ps[i].effect_kinds());
                    i += 1;
                }
                m
            }
            _ => EVENT_KINDS,
        }
    }

    /// Does the event match, for the card `me` that declares the predicate? A slot predicate that needs a
    /// checked read (`SlotPred::TypeIs`, `HasAbility`, ...) makes it: a predicate never fails open (events
    /// design, section 11 item 1).
    pub fn eval(&self, g: &mut Game, me: CardId, v: &EventView) -> R<bool> {
        Ok(match self {
            EventPred::Kind(k) => v.kind == *k,
            EventPred::Manual(m) => v.manual == *m,
            EventPred::Mode(m) => v.mode == Some(*m),
            EventPred::Source(z) => v.source == Some(*z),
            EventPred::Path(p) => v.path == Some(*p),
            EventPred::Cause(c) => c.eval(g, me, &v.cause),
            EventPred::Card(p) => v.card.map_or(false, |c| pred(g, c, p)),
            EventPred::Base(p) => v.base.map_or(false, |c| pred(g, c, p)),
            EventPred::This(Role::Card) => v.card == Some(me),
            EventPred::This(Role::Base) => v.base == Some(me),
            EventPred::This(Role::CauseCard) => v.cause.card == Some(me),
            EventPred::Slot(sp) => match v.slot {
                None => false,
                Some(s) => match slot_pred(g, me, s, sp) {
                    Some(b) => b,
                    None => slot_pred_m(g, me, s, sp)?,
                },
            },
            EventPred::Owner(w) => who_is(g, me, *w, v.owner),
            EventPred::Turn(t) => match t {
                TurnOf::EventOwner => v.turn == v.owner,
                TurnOf::NotEventOwner => v.turn != v.owner,
                TurnOf::Me => v.turn == g.st.owner(me) as u8,
                TurnOf::Opp => v.turn != g.st.owner(me) as u8,
            },
            EventPred::Condition(c) => v.condition == Some(*c),
            EventPred::All(ps) => {
                for p in ps.iter() {
                    if !p.eval(g, me, v)? {
                        return Ok(false);
                    }
                }
                true
            }
            EventPred::Any(ps) => {
                for p in ps.iter() {
                    if p.eval(g, me, v)? {
                        return Ok(true);
                    }
                }
                false
            }
            EventPred::Not(p) => !p.eval(g, me, v)?,
        })
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

    fn enter(g: &Game, c: CardId, from: RulesZone, cause: Cause, turn: u8) -> EventView {
        let p = g.st.owner(c);
        let mode = if matches!(cause.kind, CauseKind::Rule { which: RuleWhich::Action }) { EnterMode::Rule } else { EnterMode::Effect };
        EventView { source: Some(from), card: Some(c), slot: Some(bench_slot(g, p)), mode: Some(mode), ..EventView::new(EventKind::EnterPlay, cause, p as u8, turn) }
    }

    fn ev(p: &EventPred, g: &mut Game, me: CardId, v: &EventView) -> bool {
        p.eval(g, me, v).unwrap()
    }

    /// Risky Ruins: a Basic non-[D] Pokémon put onto its owner's Bench during their turn, any source
    /// (id2233). Retreating isn't EnterPlay (id2329).
    static RISKY_RUINS: EventPred = EventPred::All(&[
        EventPred::Kind(EventKind::EnterPlay),
        EventPred::Slot(SlotPred::IsBench),
        EventPred::Card(Pred::All(&[Pred::Basic, Pred::Not(&Pred::PokemonType(ct::DARK))])),
        EventPred::Turn(TurnOf::EventOwner),
    ]);

    #[test]
    fn risky_ruins() {
        let mut g = game();
        let ruins = card(&g, "Risky Ruins MEG 127", 0);
        let duskull = card(&g, "Duskull PRE 35", 1);
        let murkrow = card(&g, "Team Rocket's Murkrow DRI 127", 1);
        assert!(pred(&g, murkrow, &Pred::PokemonType(ct::DARK)));
        let own_ability = Cause::new(CauseKind::Ability, Some(duskull), 1);
        // Duskull's Come and Get You puts it from the discard pile: Risky Ruins applies.
        let v = enter(&g, duskull, RulesZone::Discard, own_ability, 1);
        assert!(ev(&RISKY_RUINS, &mut g, ruins, &v));
        let v = enter(&g, duskull, RulesZone::Hand, Cause::rule(RuleWhich::Action, 1), 1);
        assert!(ev(&RISKY_RUINS, &mut g, ruins, &v));
        // Not during the opponent's turn, not a [D] Pokémon, not a retreat.
        let v = enter(&g, duskull, RulesZone::Hand, Cause::rule(RuleWhich::Action, 1), 0);
        assert!(!ev(&RISKY_RUINS, &mut g, ruins, &v));
        let v = enter(&g, murkrow, RulesZone::Hand, Cause::rule(RuleWhich::Action, 1), 1);
        assert!(!ev(&RISKY_RUINS, &mut g, ruins, &v));
        let retreat = EventView { kind: EventKind::ChangeActive, ..enter(&g, duskull, RulesZone::Hand, Cause::rule(RuleWhich::Retreat, 1), 1) };
        assert!(!ev(&RISKY_RUINS, &mut g, ruins, &retreat));
    }

    /// Meowth ex Last-Ditch Catch: played from the hand onto the Bench.
    static MEOWTH: EventPred = EventPred::All(&[EventPred::Kind(EventKind::EnterPlay), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand), EventPred::Mode(EnterMode::Rule), EventPred::Slot(SlotPred::IsBench)]);

    #[test]
    fn meowth_ex() {
        let mut g = game();
        let meowth = card(&g, "Meowth ex POR 62", 0);
        let other = card(&g, "Meowth ex POR 62", 1);
        let rule = Cause::rule(RuleWhich::Action, 0);
        let v = enter(&g, meowth, RulesZone::Hand, rule, 0);
        assert!(ev(&MEOWTH, &mut g, meowth, &v));
        let v = enter(&g, meowth, RulesZone::Deck, rule, 0);
        assert!(!ev(&MEOWTH, &mut g, meowth, &v));
        let v = enter(&g, other, RulesZone::Hand, rule, 0);
        assert!(!ev(&MEOWTH, &mut g, meowth, &v));
        // Put from the hand by an effect isn't played by the rule.
        let v = enter(&g, meowth, RulesZone::Hand, Cause::new(CauseKind::Trainer, None, 0), 0);
        assert!(!ev(&MEOWTH, &mut g, meowth, &v));
    }

    /// Team Rocket's Arbok: no Pokémon with an Ability (except Team Rocket's) played from the hand or
    /// evolved from the hand. Rare Candy is covered; Grand Tree isn't (id1133).
    static ARBOK: EventPred = EventPred::All(&[
        EventPred::Any(&[EventPred::Kind(EventKind::EnterPlay), EventPred::Kind(EventKind::Evolve)]),
        EventPred::Source(RulesZone::Hand),
        EventPred::Card(Pred::All(&[Pred::PrintsAbility, Pred::Not(&Pred::Tag(tag::TEAM_ROCKET))])),
    ]);

    #[test]
    fn team_rockets_arbok() {
        let mut g = game();
        let arbok = card(&g, "Duskull PRE 35", 0); // the declaring card's owner is all that matters here
        let slowpoke = card(&g, "Slowpoke MEP 86", 1);
        let mewtwo = card(&g, "Team Rocket's Mewtwo ex DRI 81", 1);
        let candy = card(&g, "Rare Candy MEG 125", 1);
        assert!(pred(&g, mewtwo, &Pred::PrintsAbility) && pred(&g, mewtwo, &Pred::Tag(tag::TEAM_ROCKET)));
        let rule = Cause::rule(RuleWhich::Action, 1);
        let v = enter(&g, slowpoke, RulesZone::Hand, rule, 1);
        assert!(ev(&ARBOK, &mut g, arbok, &v));
        let v = enter(&g, mewtwo, RulesZone::Hand, rule, 1);
        assert!(!ev(&ARBOK, &mut g, arbok, &v));
        let evolve = |g: &Game, from: RulesZone, path: EvolvePath, cause: Cause| EventView { kind: EventKind::Evolve, path: Some(path), mode: None, ..enter(g, slowpoke, from, cause, 1) };
        // Rare Candy: from the hand, by a Trainer.
        let v = evolve(&g, RulesZone::Hand, EvolvePath::Rule, Cause::new(CauseKind::Trainer, Some(candy), 1));
        assert!(ev(&ARBOK, &mut g, arbok, &v));
        // Grand Tree: from the deck, by a Stadium.
        let v = evolve(&g, RulesZone::Deck, EvolvePath::Effect, Cause::new(CauseKind::Stadium, None, 1));
        assert!(!ev(&ARBOK, &mut g, arbok, &v));
        // The kinds it listens to.
        assert!(ARBOK.effect_kinds().has(crate::effects::k::ENTER_PLAY) && ARBOK.effect_kinds().has(crate::effects::k::EVOLVE));
        assert!(!ARBOK.effect_kinds().has(crate::effects::k::DEVOLVE));
        assert!(EventPred::NEVER.is_never() && EventPred::NEVER.effect_kinds() == KindMask::EMPTY);
    }

    /// Roles: Rare Candy's restriction on its own Evolve (the cause card), Eevee's permission on its own
    /// evolving (the base).
    #[test]
    fn roles() {
        let mut g = game();
        let candy = card(&g, "Rare Candy MEG 125", 1);
        let other_candy = g.st.cards.iter().enumerate().filter(|(_, c)| c.def == g.st.cards[candy as usize].def && c.owner == 1).map(|(i, _)| i as CardId).nth(1).unwrap();
        let duskull = card(&g, "Duskull PRE 35", 1);
        let slowpoke = card(&g, "Slowpoke MEP 86", 1);
        let v = EventView { kind: EventKind::Evolve, card: Some(slowpoke), base: Some(duskull), cause: Cause::new(CauseKind::Trainer, Some(candy), 1), ..EventView::new(EventKind::Evolve, Cause::rule(RuleWhich::Action, 1), 1, 1) };
        let own = EventPred::This(Role::CauseCard);
        assert!(ev(&own, &mut g, candy, &v));
        assert!(!ev(&own, &mut g, other_candy, &v));
        assert!(ev(&EventPred::This(Role::Base), &mut g, duskull, &v));
        assert!(!ev(&EventPred::This(Role::Base), &mut g, slowpoke, &v));
        assert!(ev(&EventPred::This(Role::Card), &mut g, slowpoke, &v));
        // Turn referents: the event owner's turn vs the declaring card owner's.
        let mine = card(&g, "Duskull PRE 35", 0);
        assert!(ev(&EventPred::Turn(TurnOf::EventOwner), &mut g, mine, &v));
        assert!(ev(&EventPred::Turn(TurnOf::Opp), &mut g, mine, &v));
        assert!(!ev(&EventPred::Turn(TurnOf::Me), &mut g, mine, &v));
    }

    /// Slowpoke Dopey Face: can't be Confused, whatever causes it (Lisia's Appeal included).
    static DOPEY_FACE: EventPred = EventPred::All(&[EventPred::Kind(EventKind::GainCondition), EventPred::Condition(SpecialCondition::Confused)]);
    /// Hide 'n' Sneak: prevent the effects of the opponent's attacks and Abilities.
    static HIDE_N_SNEAK: EventPred = EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Any(&[CausePred::Kind(CauseKind::Attack), CausePred::Kind(CauseKind::Ability)])]));

    #[test]
    fn slowpoke_and_hide_n_sneak() {
        let mut g = game();
        let slowpoke = card(&g, "Slowpoke MEP 86", 0);
        let lisia = Cause::new(CauseKind::Trainer, None, 1);
        let b = bench_slot(&g, 0);
        let confuse = |cause: Cause, c: SpecialCondition| EventView { slot: Some(b), condition: Some(c), ..EventView::new(EventKind::GainCondition, cause, 0, 1) };
        assert!(ev(&DOPEY_FACE, &mut g, slowpoke, &confuse(lisia, SpecialCondition::Confused)));
        assert!(!ev(&DOPEY_FACE, &mut g, slowpoke, &confuse(lisia, SpecialCondition::Poisoned)));
        let attack = Cause::attack(1, None, AttackRef { card: 0, index: 0 });
        assert!(ev(&HIDE_N_SNEAK, &mut g, slowpoke, &confuse(attack, SpecialCondition::Confused)));
        assert!(ev(&HIDE_N_SNEAK, &mut g, slowpoke, &confuse(Cause::new(CauseKind::Ability, None, 1), SpecialCondition::Confused)));
        assert!(!ev(&HIDE_N_SNEAK, &mut g, slowpoke, &confuse(lisia, SpecialCondition::Confused)), "a Trainer isn't an attack or an Ability");
        assert!(!ev(&HIDE_N_SNEAK, &mut g, slowpoke, &confuse(Cause::attack(0, None, AttackRef { card: 0, index: 0 }), SpecialCondition::Confused)), "its own side's attack");
    }

    /// A CoinFlip has no card: a lock or prevention naming a card (`EventPred::Card`) never matches it, whatever the
    /// card predicate; its cause's card is matched through `EventPred::Cause`.
    #[test]
    fn coin_flip_has_no_card() {
        let mut g = game();
        let me = card(&g, "Slowpoke MEP 86", 0);
        let cause = Cause::new(crate::cause::CauseKind::Trainer, Some(me), 0);
        let v = crate::engine::condition::coin_view(&g, 0, CoinPurpose::Effect, true, cause);
        assert_eq!(v.card, None);
        assert!(!EventPred::Card(Pred::Any).eval(&mut g, me, &v).unwrap());
        assert!(EventPred::Kind(EventKind::CoinFlip).eval(&mut g, me, &v).unwrap());
    }
}
