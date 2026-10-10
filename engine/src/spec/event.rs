//! Rules events as data, and the one predicate type over them (docs/design/events-design.md, sections 4
//! and 5): triggers, locks, prevention, permissions and restrictions are all `EventPred`s.
//!
//! Events batch 2: EnterPlay, Evolve, Devolve and Swap are built by their routines (`engine/enter.rs`); events
//! batch 3: Attach, MoveEnergy and MoveTool by theirs (`engine/attach.rs`); events batch 4: GainCondition,
//! RemoveCondition, RemoveCounters (healing) and CoinFlip by theirs (`engine/condition.rs`); events batch 5:
//! ChangeActive by its routine (`engine/change_active.rs`). They are read by triggers (`trigger::Event::On`), locks (`LockDecl::forbids`), permissions (`Modifier::Permit`)
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
    /// A lasting effect is put on a Pokémon or a player by an attack ("during your opponent's next turn, ..."; events
    /// batch 6, user decision D4): what "prevent all effects of attacks done to this Pokémon" stops (APR C-17).
    ApplyEffect,
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
            EventKind::ChangeActive => Some(k::CHANGE_ACTIVE),
            EventKind::Damage => Some(k::DAMAGE),
            EventKind::PlaceCounters => Some(k::PLACE_COUNTERS),
            EventKind::MoveCounters => Some(k::MOVE_COUNTERS_EVENT),
            EventKind::KnockOut => Some(k::KNOCK_OUT),
            EventKind::LeavePlay => Some(k::LEAVE_PLAY),
            EventKind::TakePrizes => Some(k::TAKE_PRIZES),
            EventKind::ApplyEffect => Some(k::APPLY_EFFECT),
            EventKind::Discard => Some(k::DISCARD),
            EventKind::PutIntoHand => Some(k::PUT_INTO_HAND),
            EventKind::PutIntoDeck => Some(k::PUT_INTO_DECK),
            EventKind::Draw => Some(k::DRAW),
            EventKind::PlayTrainer => Some(k::PLAY_TRAINER),
            EventKind::UseAttack => Some(k::USE_ATTACK),
            EventKind::UseAbility => Some(k::USE_POWER),
            EventKind::UseStadium => Some(k::USE_STADIUM),
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

/// How a Trainer card's effect comes to be used (the PlayTrainer event; APR B-01..B-04, E-26).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrainerUse {
    /// Played from the hand by the game rule (APR B-01 step 2: chosen from the hand and revealed): the turn's limits
    /// apply (one Supporter, one Stadium), "can't play ... from their hand" locks read it, and it counts for "if you
    /// played a Supporter card from your hand this turn" (E-26).
    Played,
    /// Its effect is used by another card without playing it (Mr. Mime's Look-Alike Show: "use the effect of a Supporter
    /// card you find there as the effect of this attack"): it keeps its printed conditions and none of the play rules
    /// (id2225, id2226, id2376).
    Used,
}

/// Where cards put into a deck go (the PutIntoDeck event; APR E-35, E-36).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeckPosition {
    /// On top ("put them on top of your deck").
    Top,
    /// On the bottom ("put them on the bottom of your deck"; a plain "put them into your deck" the text doesn't shuffle).
    Bottom,
    /// Shuffled in ("shuffle them into your deck", APR E-36): the cards go in; the deck's shuffle is the printed one.
    ShuffledIn,
}

/// How the Active Pokémon changes (the ChangeActive event), named from the rulebook. Which Pokémon the change is
/// done to (the event's card and spot) follows from it: the Active Pokémon for a retreat, a switch and a switch-out,
/// the Benched Pokémon brought in for a switch-in, and a promotion (APR C-04 / C-05 specific
/// cases; id42, id2025, id2155; the official JP Q&A on Hariyama's Heave-Ho Catcher).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveChange {
    /// The player retreats their Active Pokémon (the turn action; APR A-03).
    Retreat,
    /// "Switch your Active Pokémon with 1 of your Benched Pokémon" / "switch this Pokémon with 1 of your Benched
    /// Pokémon" (APR C-03): the side's own player switches, by their own effect.
    Switch,
    /// "Switch in 1 of your opponent's Benched Pokémon to the Active Spot" / "switch 1 of your opponent's Benched
    /// Pokémon with their Active Pokémon" (APR C-05): an effect done to the Benched Pokémon chosen.
    SwitchIn,
    /// "Switch out your opponent's Active Pokémon to the Bench. (Your opponent chooses the new Active Pokémon.)" /
    /// "your opponent switches their Active Pokémon with 1 of their Benched Pokémon" (APR C-04): an effect done to
    /// the Active Pokémon.
    SwitchOut,
    /// A Benched Pokémon is promoted to the empty Active Spot (after the Active Pokémon left play).
    Promotion,
}

impl ActiveChange {
    /// Is the change done to the Pokémon leaving the Active Spot (otherwise to the one coming in)?
    pub const fn done_to_leaving(self) -> bool {
        matches!(self, ActiveChange::Retreat | ActiveChange::Switch | ActiveChange::SwitchOut)
    }
}

/// Which end of a move an event view is about (MoveCounters; MoveEnergy / MoveTool when their preventions are read
/// per end): the Pokémon the counters or the card leave (`From`) or the one they go onto (`To`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveEnd {
    From,
    To,
}

/// How a Pokémon is Knocked Out (the KnockOut event; APR D, E-04).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KoBy {
    /// It took damage from the opponent's attack in progress ("Knocked Out by damage from an attack").
    AttackDamage,
    /// An effect Knocks it Out ("is Knocked Out", "Knock Out ..."): recorded, then Knocked Out at the next state
    /// check with the others (id2089, id810).
    Effect,
    /// Its HP reached 0 otherwise (damage counters, a Special Condition, an HP change).
    Other,
}

/// A party to an event, relative to the event's owner (the player whose Pokémon or card it is about).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Party {
    EventOwner,
    NotEventOwner,
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
/// the declaring card's owner). Every one but `NotEventOwner` is false while it is nobody's turn ([`NO_TURN`]:
/// Pokémon Checkup, APR F).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TurnOf {
    /// The turn of the player whose card / Pokémon the event is about ("during their turn").
    EventOwner,
    /// Not that player's turn (the other player's, or nobody's: Pokémon Checkup).
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
    /// The causing card is a Pokémon in play whose spot matches (a checked read where the predicate needs one; false
    /// when the card is not a Pokémon in play): "attacks from Pokémon that have an Ability / Basic / Evolution / ex".
    Pokemon(SlotPred),
    /// The cause is the attack of this name (Unown's own Mysterious Signal).
    Attack(&'static str),
    /// The causing card is the Pokémon that holds the declaring card ("the Pokémon this card is attached to": Backtrack
    /// Badge's "an attack of the [C] Pokémon this card is attached to"). False when the declaring card isn't attached to
    /// a Pokémon.
    Holder,
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
    /// ChangeActive: how the Active Pokémon changes.
    Change(ActiveChange),
    /// ChangeActive: the spot of the Pokémon leaving the Active Spot (for the Bench) matches: "this Pokémon can't
    /// retreat" is `Change(Retreat) & From(IsThisPokemon)`, "when this Pokémon moves from the Active Spot to the Bench"
    /// `Kind(ChangeActive) & From(IsThisPokemon)`. False for a promotion (nothing leaves).
    From(SlotPred),
    /// ChangeActive: the spot of the Pokémon moving to the Active Spot matches ("when this Pokémon moves from your
    /// Bench to the Active Spot": `Kind(ChangeActive) & To(IsThisPokemon)`). False while the Pokémon isn't chosen yet
    /// (a switch-out checked before the opponent chooses).
    To(SlotPred),
    /// Who causes the event (the `Cause` player) relative to the event's owner: Battle Cage's "from the opponent's
    /// Pokémon" is `Actor(NotEventOwner)` (whose Bench it is decides, not whose card Battle Cage is).
    Actor(Party),
    /// Which end of a move the view is about (MoveCounters: the Pokémon the counters leave or go onto).
    End(MoveEnd),
    /// How a Pokémon is Knocked Out (KnockOut).
    KoBy(KoBy),
    /// Where the cards go (LeavePlay, Devolve, TakePrizes).
    Dest(RulesZone),
    /// KnockOut: the attack's damage was done to the Pokémon while it was in the Active Spot ("if this Pokémon is in the
    /// Active Spot and is Knocked Out by damage from an attack": the Active Spot is read when the damage is done, id1992).
    DamagedActive,
    /// PlayTrainer: played from the hand by the rule, or used by another card's effect ("can't play ... from their hand"
    /// is `Use(Played) & Source(Hand)`, one conjunct per clause).
    Use(TrainerUse),
    /// The cause's card is the Pokémon in the event's spot (its top Pokémon card): "discarded by an effect of an attack of
    /// the Pokémon it is attached to" (Boomerang Energy), read on the spot the card left.
    CauseOnSlot,
    /// CoinFlip: what the coin is flipped for.
    Purpose(CoinPurpose),
    /// UseAttack: the attacking Pokémon provides at most this many Energy (Walrein's Frigid Fangs: "Pokémon that have 2
    /// or less Energy attached can't attack"; the count of a checked read of the Energy it provides, which the
    /// UseAttack's checks make only when a lock on the player reads it: [`EventPred::reads_energy`]).
    EnergyAtMost(i32),
    All(&'static [EventPred]),
    Any(&'static [EventPred]),
    Not(&'static EventPred),
}

/// No player's turn: [`EventView::turn`] during Pokémon Checkup and the game's setup.
pub const NO_TURN: u8 = 2;

/// The player whose turn it is, as an event records it: nobody's during Pokémon Checkup ("Pokémon Checkup takes
/// place in neither player's turn", APR F) and during setup, so "during your turn" / "during their turn" is false
/// there for every event (a Pokémon promoted after a Knock Out at Checkup isn't moved during its owner's turn).
#[inline]
pub fn whose_turn(g: &Game) -> u8 {
    use crate::types::GamePhase;
    match g.st.phase {
        GamePhase::BetweenTurns | GamePhase::Setup | GamePhase::WaitingForPlayers => NO_TURN,
        _ => g.st.active_player,
    }
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
    /// The player whose turn it is ([`whose_turn`]; [`NO_TURN`] during Pokémon Checkup and setup).
    pub turn: u8,
    /// Evolve: the Pokémon evolved from came into play this turn (the slot's entered-turn record,
    /// `Slot::entered_turn`, which no permission rewrites).
    pub base_entered_this_turn: bool,
    /// Evolve: it is the owner's first turn (the game's turns 1 and 2 are each player's first turn).
    pub owner_first_turn: bool,
    /// ChangeActive: how it changes, the Active Spot (`from`, empty for a promotion) and the Benched Pokémon coming
    /// in (`to`; `None` before it is chosen). `card` / `slot` are the Pokémon the change is done to
    /// (`ActiveChange::done_to_leaving`).
    pub change: Option<ActiveChange>,
    pub from: Option<SlotRef>,
    pub to: Option<SlotRef>,
    /// MoveCounters (MoveEnergy / MoveTool per end): which end of the move this view is about; `slot` is that end's spot.
    pub end: Option<MoveEnd>,
    /// KnockOut: how the Pokémon is Knocked Out.
    pub ko_by: Option<KoBy>,
    /// LeavePlay, Devolve, TakePrizes: where the cards go.
    pub dest: Option<RulesZone>,
    /// Damage: the attack isn't affected by effects on the damaged Pokémon (Shred, done to the opponent's Pokémon):
    /// its preventions are skipped (APR C-16; RULES.md "Shred").
    pub ignores_defender: bool,
    /// KnockOut: the attack in progress damaged the Pokémon while it was in the Active Spot.
    pub damaged_active: bool,
    /// PlayTrainer: played from the hand or used by another card's effect.
    pub trainer_use: Option<TrainerUse>,
    /// PutIntoDeck: where the cards go.
    pub position: Option<DeckPosition>,
    /// UseAttack: the attack used (its name is what "can't use [Attack Name]" reads, APR C-15).
    pub attack: Option<crate::state::AttackRef>,
    /// UseAttack: the Energy the attacking Pokémon provides, when a lock reads it ([`EventPred::EnergyAtMost`]).
    pub energy: Option<i32>,
}

impl EventView {
    /// An event of `kind` about `owner`'s card, with nothing else set.
    pub const fn new(kind: EventKind, cause: Cause, owner: u8, turn: u8) -> EventView {
        EventView { kind, source: None, mode: None, manual: false, path: None, cause, card: None, base: None, slot: None, condition: None, amount: 0, purpose: None, heads: None, owner, turn, base_entered_this_turn: false, owner_first_turn: false, change: None, from: None, to: None, end: None, ko_by: None, dest: None, ignores_defender: false, damaged_active: false, trainer_use: None, position: None, attack: None, energy: None }
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

/// Does the spot match, with a checked read when the predicate needs one (never fails open)?
fn slot_matches(g: &mut Game, me: CardId, s: SlotRef, sp: &SlotPred) -> R<bool> {
    Ok(match slot_pred(g, me, s, sp) {
        Some(b) => b,
        None => slot_pred_m(g, me, s, sp)?,
    })
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

    /// Does the cause match, for the card `me` that declares the predicate? A spot predicate on the causing Pokémon
    /// (`Pokemon`) makes a checked read where it needs one (never fails open).
    pub fn eval(&self, g: &mut Game, me: CardId, c: &Cause) -> R<bool> {
        Ok(match self {
            CausePred::Kind(k) => c.kind == *k,
            CausePred::Rule => matches!(c.kind, CauseKind::Rule { .. }),
            CausePred::By(w) => who_is(g, me, *w, c.player),
            CausePred::Card(p) => c.card.map_or(false, |x| pred(g, x, p)),
            CausePred::Pokemon(sp) => match c.card.and_then(|x| g.st.find_pokemon_slot(x)) {
                Some((q, s)) if g.st.slot_pokemon(q, s) == c.card => slot_matches(g, me, SlotRef::new(q, s), sp)?,
                _ => false,
            },
            CausePred::Attack(name) => c.attack.map_or(false, |a| crate::engine::attack::attack_def(g, a).name == *name),
            CausePred::Holder => match (c.card, g.st.find_pokemon_slot(me)) {
                (Some(x), Some((q, s))) => x != me && g.st.slot_pokemon(q, s) == Some(x),
                _ => false,
            },
            CausePred::All(ps) => {
                for p in ps.iter() {
                    if !p.eval(g, me, c)? {
                        return Ok(false);
                    }
                }
                true
            }
            CausePred::Any(ps) => {
                for p in ps.iter() {
                    if p.eval(g, me, c)? {
                        return Ok(true);
                    }
                }
                false
            }
            CausePred::Not(p) => !p.eval(g, me, c)?,
        })
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
    crate::effects::k::CHANGE_ACTIVE,
    crate::effects::k::PLACE_COUNTERS,
    crate::effects::k::MOVE_COUNTERS_EVENT,
    crate::effects::k::DAMAGE,
    crate::effects::k::KNOCK_OUT,
    crate::effects::k::LEAVE_PLAY,
    crate::effects::k::TAKE_PRIZES,
    crate::effects::k::APPLY_EFFECT,
    crate::effects::k::DISCARD,
    crate::effects::k::PUT_INTO_HAND,
    crate::effects::k::PUT_INTO_DECK,
    crate::effects::k::DRAW,
    crate::effects::k::PLAY_TRAINER,
    crate::effects::k::USE_ATTACK,
    crate::effects::k::USE_POWER,
    crate::effects::k::USE_STADIUM,
]);
/// The events with an effect done to a Pokémon or its cards, which a `Prevent` naming no `Kind` ranges over ("prevent all
/// effects of attacks done to X"): an explicit list (events batch 7). Not Damage: damage is not an effect (APR C-17
/// "(Damage is not an effect)", B-08 / B-09; id2289, id2333, id2398; a prevention of damage names `Kind(Damage)`). Not
/// the events that aren't done to a Pokémon (user decision D1: Discard and Draw are cards from the hand or the deck,
/// PutIntoHand / PutIntoDeck cards from the deck, the discard pile or the Prizes; a card leaving a Pokémon is that
/// Pokémon's LeavePlay), nor PlayTrainer and the turn actions: a cause-only prevention can't match them.
pub const EFFECT_EVENT_KINDS: KindMask = crate::effects::mask(&[
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
    crate::effects::k::CHANGE_ACTIVE,
    crate::effects::k::PLACE_COUNTERS,
    crate::effects::k::MOVE_COUNTERS_EVENT,
    crate::effects::k::KNOCK_OUT,
    crate::effects::k::LEAVE_PLAY,
    crate::effects::k::TAKE_PRIZES,
    crate::effects::k::APPLY_EFFECT,
]);
/// The card-movement events of events batch 7 (Discard, PutIntoHand, PutIntoDeck, Draw): a lock over them sets
/// `DECLARES_CARD_LOCK` (Poké Vital A, Neutralization Zone). No `Prevent` ranges over them (they aren't done to a Pokémon).
pub const CARD_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::DISCARD, crate::effects::k::PUT_INTO_HAND, crate::effects::k::PUT_INTO_DECK, crate::effects::k::DRAW]);
/// PlayTrainer (events batch 7): a lock over it sets `DECLARES_PLAY_LOCK`. No `Prevent` ranges over it (it isn't done to
/// a Pokémon; a used Supporter's effects are the attack's, on the Pokémon they reach: id2025).
pub const PLAY_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::PLAY_TRAINER]);
/// The turn actions UseAttack, UseAbility, UseStadium (events batch 7): a lock over them sets `DECLARES_USE_LOCK`.
pub const USE_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::USE_ATTACK, crate::effects::k::USE_POWER, crate::effects::k::USE_STADIUM]);

/// `m` without the kind `k`.
pub const fn without(m: KindMask, k: u32) -> KindMask {
    let mut out = m;
    out.0[(k >> 6) as usize] &= !(1u64 << (k & 63));
    out
}
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
/// ChangeActive (events batch 5): `DECLARES_ACTIVE_LOCK` / `DECLARES_ACTIVE_PREVENT`.
pub const ACTIVE_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::CHANGE_ACTIVE]);
/// The events no card handles when they are dispatched (events batches 4, 5 and 6): what reacts to them is a declaration
/// read through the dispatch index, by the triggers after the event (`run::after_event`) and by the locks and
/// preventions the event's routine asks. `Game::reduce_effect` doesn't call the cards for them (events design,
/// section 9). The batch 2 and 3 events still have dispatch handlers (once-per-turn markers, attach guards).
pub const INDEX_ONLY_EVENT_KINDS: KindMask = CARD_EVENT_KINDS.or(CONDITION_EVENT_KINDS).or(HEAL_EVENT_KINDS).or(COIN_EVENT_KINDS).or(ACTIVE_EVENT_KINDS).or(COUNTER_EVENT_KINDS).or(LEAVE_EVENT_KINDS).or(crate::effects::mask(&[crate::effects::k::TAKE_PRIZES])).or(APPLY_EVENT_KINDS);
/// PlaceCounters and MoveCounters (events batch 6): `DECLARES_COUNTER_LOCK` (Patrat's Watchful Eye) / `DECLARES_COUNTER_PREVENT`.
pub const COUNTER_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::PLACE_COUNTERS, crate::effects::k::MOVE_COUNTERS_EVENT]);
/// Damage (events batch 6): `DECLARES_DAMAGE_PREVENT` (no lock: no text says a Pokémon can't be damaged).
pub const DAMAGE_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::DAMAGE]);
/// KnockOut: `DECLARES_KO_PREVENT` (asked for a Knock Out by an effect only; the state check's is the rule's).
pub const KO_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::KNOCK_OUT]);
/// LeavePlay: `DECLARES_LEAVE_PREVENT` (asked for an effect's removal only).
pub const LEAVE_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::LEAVE_PLAY]);
/// Evolve, Devolve and Swap: a `Prevent` over them sets `DECLARES_POKEMON_PREVENT` (EnterPlay has no Pokémon to protect yet).
pub const POKEMON_PREVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::EVOLVE, crate::effects::k::DEVOLVE, crate::effects::k::SWAP]);
/// ApplyEffect (user decision D4): `DECLARES_APPLY_PREVENT`.
pub const APPLY_EVENT_KINDS: KindMask = crate::effects::mask(&[crate::effects::k::APPLY_EFFECT]);

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
            EventPred::Cause(_) | EventPred::This(_) | EventPred::CauseOnSlot => true,
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

    /// Does the predicate read the Energy the event's Pokémon provides ([`EventPred::EnergyAtMost`], a checked read the
    /// UseAttack's checks make only for a lock that needs it)?
    pub const fn reads_energy(&self) -> bool {
        match self {
            EventPred::EnergyAtMost(_) => true,
            EventPred::All(ps) | EventPred::Any(ps) => {
                let mut i = 0;
                while i < ps.len() {
                    if ps[i].reads_energy() {
                        return true;
                    }
                    i += 1;
                }
                false
            }
            EventPred::Not(p) => p.reads_energy(),
            _ => false,
        }
    }

    /// The effect kinds whose events can match: the kinds the predicate names, or every event kind with an
    /// effect where it names none. The declaring card's dispatch mask.
    pub const fn effect_kinds(&self) -> KindMask {
        self.kinds_over(EVENT_KINDS)
    }

    /// The effect kinds a `Prevent` over this predicate ranges over: the kinds it names, or every event kind with an
    /// effect except Damage where it names none ([`EFFECT_EVENT_KINDS`]: "prevent all effects of attacks" never
    /// prevents damage). The `Prevent` reader tests the event's kind against it before evaluating the predicate.
    pub const fn prevent_kinds(&self) -> KindMask {
        self.kinds_over(EFFECT_EVENT_KINDS)
    }

    /// The kinds the predicate names, `base` where it names none.
    const fn kinds_over(&self, base: KindMask) -> KindMask {
        match self.named_kinds(base) {
            Some(m) => m,
            None => base,
        }
    }

    /// The kinds the predicate restricts the event to, `None` when it names none (it then ranges over `base`). A
    /// conjunction is limited by each member that names kinds (a member that names none doesn't restrict it); a
    /// disjunction ranges over the union, a member naming none counting as `base`: "damage from and effects of attacks"
    /// is `All[Any[Kind(Damage), ALWAYS], Cause(..)]`, over Damage and every effect.
    const fn named_kinds(&self, base: KindMask) -> Option<KindMask> {
        match self {
            EventPred::Kind(e) => Some(match e.effect_kind() {
                Some(x) => crate::effects::mask(&[x]),
                None => KindMask::EMPTY,
            }),
            EventPred::All(ps) => {
                let mut i = 0;
                let mut m: Option<KindMask> = None;
                while i < ps.len() {
                    if let Some(x) = ps[i].named_kinds(base) {
                        m = Some(match m {
                            Some(y) => KindMask([y.0[0] & x.0[0], y.0[1] & x.0[1], y.0[2] & x.0[2], y.0[3] & x.0[3]]),
                            None => x,
                        });
                    }
                    i += 1;
                }
                m
            }
            EventPred::Any(ps) => {
                let mut i = 0;
                let mut m = KindMask::EMPTY;
                while i < ps.len() {
                    m = m.or(ps[i].kinds_over(base));
                    i += 1;
                }
                Some(m)
            }
            _ => None,
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
            EventPred::Cause(c) => c.eval(g, me, &v.cause)?,
            EventPred::Card(p) => v.card.map_or(false, |c| pred(g, c, p)),
            EventPred::Base(p) => v.base.map_or(false, |c| pred(g, c, p)),
            EventPred::This(Role::Card) => v.card == Some(me),
            EventPred::This(Role::Base) => v.base == Some(me),
            EventPred::This(Role::CauseCard) => v.cause.card == Some(me),
            EventPred::Slot(sp) => match v.slot {
                None => false,
                Some(s) => slot_matches(g, me, s, sp)?,
            },
            EventPred::Owner(w) => who_is(g, me, *w, v.owner),
            EventPred::Turn(t) => match t {
                TurnOf::EventOwner => v.turn == v.owner,
                TurnOf::NotEventOwner => v.turn != v.owner,
                TurnOf::Me => v.turn == g.st.owner(me) as u8,
                TurnOf::Opp => v.turn == 1 - g.st.owner(me) as u8,
            },
            EventPred::Condition(c) => v.condition == Some(*c),
            EventPred::Change(c) => v.change == Some(*c),
            EventPred::From(sp) => match v.from {
                Some(s) => slot_matches(g, me, s, sp)?,
                None => false,
            },
            EventPred::To(sp) => match v.to {
                Some(s) => slot_matches(g, me, s, sp)?,
                None => false,
            },
            EventPred::Actor(Party::EventOwner) => v.cause.player == v.owner,
            EventPred::Actor(Party::NotEventOwner) => v.cause.player != v.owner,
            EventPred::End(e) => v.end == Some(*e),
            EventPred::KoBy(k) => v.ko_by == Some(*k),
            EventPred::Dest(z) => v.dest == Some(*z),
            EventPred::DamagedActive => v.damaged_active,
            EventPred::Use(u) => v.trainer_use == Some(*u),
            EventPred::CauseOnSlot => match (v.cause.card, v.slot) {
                (Some(c), Some(s)) => g.st.slot_pokemon(s.p as usize, s.s) == Some(c),
                _ => false,
            },
            EventPred::Purpose(p) => v.purpose == Some(*p),
            EventPred::EnergyAtMost(n) => v.energy.map_or(false, |e| e <= *n),
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
        // Pokémon Checkup is in neither player's turn (APR F).
        let checkup = EventView { turn: NO_TURN, ..v };
        for t in [TurnOf::EventOwner, TurnOf::Me, TurnOf::Opp] {
            assert!(!ev(&EventPred::Turn(t), &mut g, mine, &checkup), "{t:?} at Checkup");
        }
        assert!(ev(&EventPred::Turn(TurnOf::NotEventOwner), &mut g, mine, &checkup));
        g.st.phase = crate::types::GamePhase::BetweenTurns;
        assert_eq!(whose_turn(&g), NO_TURN);
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

    /// "Prevent all effects of attacks" names no kind: it ranges over every event with an effect except Damage (APR C-17);
    /// a prevention of damage names `Kind(Damage)`.
    #[test]
    fn effects_are_not_damage() {
        let effects = EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(CauseKind::Attack)]));
        let k = effects.prevent_kinds();
        assert!(!k.has(crate::effects::k::DAMAGE));
        assert!(k.has(crate::effects::k::GAIN_CONDITION) && k.has(crate::effects::k::CHANGE_ACTIVE));
        let both = EventPred::Any(&[EventPred::Kind(EventKind::Damage), EventPred::Cause(CausePred::By(Who::Opp))]);
        assert!(both.prevent_kinds().has(crate::effects::k::DAMAGE) && both.prevent_kinds().has(crate::effects::k::HEAL));
        assert!(EventPred::Kind(EventKind::Damage).prevent_kinds() == crate::effects::mask(&[crate::effects::k::DAMAGE]));
        assert!(!EFFECT_EVENT_KINDS.has(crate::effects::k::DAMAGE));
    }

    /// The batch 6 attributes: who causes the event relative to its owner, the end of a move, how a Pokémon is Knocked
    /// Out, where cards go; the cause's attack by name.
    #[test]
    fn batch_6_attributes() {
        let mut g = game();
        let me = card(&g, "Slowpoke MEP 86", 0);
        let opp_attack = Cause::attack(1, None, AttackRef { card: me, index: 0 });
        let name = crate::engine::attack::attack_def(&g, AttackRef { card: me, index: 0 }).name;
        let v = EventView { end: Some(MoveEnd::To), ko_by: Some(KoBy::Effect), dest: Some(RulesZone::Discard), ..EventView::new(EventKind::PlaceCounters, opp_attack, 0, 1) };
        assert!(ev(&EventPred::Actor(Party::NotEventOwner), &mut g, me, &v));
        assert!(!ev(&EventPred::Actor(Party::EventOwner), &mut g, me, &v));
        assert!(ev(&EventPred::End(MoveEnd::To), &mut g, me, &v) && !ev(&EventPred::End(MoveEnd::From), &mut g, me, &v));
        assert!(ev(&EventPred::KoBy(KoBy::Effect), &mut g, me, &v) && !ev(&EventPred::KoBy(KoBy::AttackDamage), &mut g, me, &v));
        assert!(ev(&EventPred::Dest(RulesZone::Discard), &mut g, me, &v));
        assert!(CausePred::Attack(name).eval(&mut g, me, &opp_attack).unwrap());
        assert!(!CausePred::Attack("No Such Attack").eval(&mut g, me, &opp_attack).unwrap());
        // The cause's card is not a Pokémon in play: `Pokemon(..)` is false whatever the spot predicate.
        assert!(!CausePred::Pokemon(SlotPred::Any).eval(&mut g, me, &opp_attack).unwrap());
    }

    /// The batch 7 attributes: played vs used (PlayTrainer), the cause's card in the event's spot (Boomerang Energy's "an
    /// attack of the Pokémon it is attached to"), the coin's purpose, the Pokémon holding the declaring card (Backtrack
    /// Badge's "the Pokémon this card is attached to").
    #[test]
    fn batch_7_attributes() {
        let mut g = game();
        let sc = serde_json::json!({
            "me": {"reset": true, "active": "Slowpoke MEP 86", "active_energy": ["Psychic Energy MEE 5"]},
            "opp": {"reset": true, "active": "Duskull PRE 35"}
        });
        crate::scenario::apply(&mut g, &sc).unwrap();
        let me = g.st.active_player as usize;
        let a = g.st.players[me].active;
        let slowpoke = g.st.slot_pokemon(me, a).unwrap();
        let energy = g.st.slot(me, a).cards.iter().find(|&c| g.st.cdef(c).is_energy()).unwrap();
        let o = 1 - me;
        let duskull = g.st.slot_pokemon(o, g.st.players[o].active).unwrap();
        let attack = |card| Cause::attack(me as u8, Some(card), AttackRef { card, index: 0 });
        let left = |cause| EventView { card: Some(energy), slot: Some(SlotRef::new(me, a)), ..EventView::new(EventKind::LeavePlay, cause, me as u8, me as u8) };
        assert!(ev(&EventPred::CauseOnSlot, &mut g, energy, &left(attack(slowpoke))));
        assert!(!ev(&EventPred::CauseOnSlot, &mut g, energy, &left(attack(duskull))), "another Pokémon's attack");
        assert!(!ev(&EventPred::CauseOnSlot, &mut g, energy, &EventView { slot: None, ..left(attack(slowpoke)) }));
        // Holder: the attacking Pokémon holds the declaring card.
        assert!(CausePred::Holder.eval(&mut g, energy, &attack(slowpoke)).unwrap());
        assert!(!CausePred::Holder.eval(&mut g, energy, &attack(duskull)).unwrap());
        assert!(!CausePred::Holder.eval(&mut g, slowpoke, &attack(slowpoke)).unwrap(), "a Pokémon doesn't hold itself");
        // Played vs used.
        let play = |u| EventView { trainer_use: Some(u), source: Some(RulesZone::Hand), ..EventView::new(EventKind::PlayTrainer, Cause::rule(RuleWhich::Action, 0), 0, 0) };
        assert!(ev(&EventPred::Use(TrainerUse::Played), &mut g, slowpoke, &play(TrainerUse::Played)));
        assert!(!ev(&EventPred::Use(TrainerUse::Played), &mut g, slowpoke, &play(TrainerUse::Used)));
        // A coin's purpose.
        let v = crate::engine::condition::coin_view(&g, 0, CoinPurpose::Confusion, false, Cause::rule(RuleWhich::Action, 0));
        assert!(ev(&EventPred::Purpose(CoinPurpose::Confusion), &mut g, slowpoke, &v) && !ev(&EventPred::Purpose(CoinPurpose::Effect), &mut g, slowpoke, &v));
        assert!(EventPred::CauseOnSlot.reads_cause());
    }

    /// The card-movement events aren't done to a Pokémon (user decision D1): a cause-only prevention never ranges over
    /// them; a lock over them is listed under their kinds.
    #[test]
    fn card_events_are_not_effects_on_a_pokemon() {
        for k in [EventKind::Discard, EventKind::PutIntoHand, EventKind::PutIntoDeck, EventKind::Draw] {
            let x = k.effect_kind().unwrap();
            assert!(EVENT_KINDS.has(x) && !EFFECT_EVENT_KINDS.has(x) && CARD_EVENT_KINDS.has(x), "{k:?}");
        }
        assert!(EFFECT_EVENT_KINDS.has(crate::effects::k::LEAVE_PLAY) && !EFFECT_EVENT_KINDS.has(crate::effects::k::DAMAGE));
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
