//! The turn actions as events (events batch 7; docs/design/events-design.md, section 4 "Turn actions"; APR A-01, A-02,
//! B-04, C-15, D-01, E-12..E-14): UseAttack, UseAbility, UseStadium. Each has one check function, which execution and
//! legality (`legal.rs`) both call, so the two can't drift:
//!
//! - [`attack_checks`]: the first turn and the Special Conditions that stop an attack (`attack::can_attack_pre`), the
//!   lasting "can't attack" effects on the attacking Pokémon and its player (`attack::can_attack_post`; B7-OLD -> B8:
//!   stored as slot / player fields until batch 8 makes them lasting locks with a timing), the locks over UseAttack
//!   (`derived::event_locked`: "this Pokémon can't attack unless ..." is `LockDecl::on(Kind(UseAttack) & This(Card))` with
//!   the condition in `LockWhile::Unless`), then the cost against the Energy the Pokémon provides.
//! - [`ability_checks`]: the printed use-from flags, the Ability is there (`derived::has_no_ability`: a locked Ability
//!   doesn't exist, RULES.md "Has no Abilities"; B7-OLD -> B8: the `Power` probe behind it), the locks over UseAbility (no
//!   pool card prints "can't use Abilities").
//! - [`stadium_checks`]: once per turn, a Stadium that declares a use (`CardSpec::use_stadium`), the locks over
//!   UseStadium.
//!
//! The events keep today's effects (`Effect::UseAttack`, `UsePower`, `UseStadium`): their kinds are the events'
//! (`k::USE_ATTACK`, `k::USE_POWER`, `k::USE_STADIUM`); a Confused Pokémon's tails is an attack that doesn't happen (APR
//! D-01; ruling 857: the attack isn't used).

use crate::cause::{Cause, RuleWhich};
use crate::effects::{PowerRef, SlotRef};
use crate::game::{Game, R};
use crate::list::*;
use crate::spec::event::{EventKind, EventView};
use crate::state::AttackRef;

/// The UseAttack event of player `p`'s attack from the Pokémon in `attacking` (its top card: the copycat of a copied
/// attack), as predicates read it.
pub fn attack_view(g: &Game, p: usize, attacking: SlotRef) -> EventView {
    let cause = Cause::rule(RuleWhich::Action, p as u8);
    EventView { card: g.st.slot_pokemon(attacking.p as usize, attacking.s), slot: Some(attacking), ..EventView::new(EventKind::UseAttack, cause, p as u8, crate::spec::event::whose_turn(g)) }
}

/// The UseAbility event of player `p`'s Ability `power` of `card` (a Pokémon in play, or a card in the hand or the
/// discard pile for an Ability used from there).
pub fn ability_view(g: &Game, p: usize, power: PowerRef, card: CardId) -> EventView {
    let cause = Cause::new(crate::cause::CauseKind::Ability, Some(card), p as u8);
    let slot = g.st.find_pokemon_slot(card).filter(|(q, s)| g.st.slot_pokemon(*q, *s) == Some(card)).map(|(q, s)| SlotRef::new(q, s));
    let _ = power;
    EventView { card: Some(card), slot, ..EventView::new(EventKind::UseAbility, cause, p as u8, crate::spec::event::whose_turn(g)) }
}

/// The UseStadium event of player `p` using the Stadium `stadium` in play (the event's owner: the Stadium's owner).
pub fn stadium_view(g: &Game, p: usize, stadium: CardId) -> EventView {
    let cause = Cause::rule(RuleWhich::Action, p as u8);
    EventView { card: Some(stadium), ..EventView::new(EventKind::UseStadium, cause, g.st.owner(stadium) as u8, crate::spec::event::whose_turn(g)) }
}

/// Where the checks of a turn action read the game: execution on the game, legality on its scratch game behind its
/// plain-read gates.
pub trait ActionChecks {
    fn game(&self) -> &Game;
    /// The one lock query (`derived::event_locked`).
    fn event_locked(&mut self, v: &EventView) -> R<Option<&'static str>>;
    /// The Energy count the Pokémon in `slot` provides (a checked read; asked only under a "can't attack with this much
    /// Energy" effect).
    fn energy_count(&mut self, slot: SlotRef) -> R<i32>;
    /// Can the Pokémon in `slot` pay the cost of `attack` (the checked cost against the Energy it provides)?
    fn payable(&mut self, attack: AttackRef, slot: SlotRef) -> R<bool>;
}

impl ActionChecks for Game {
    fn game(&self) -> &Game {
        self
    }
    fn event_locked(&mut self, v: &EventView) -> R<Option<&'static str>> {
        crate::derived::event_locked(self, v)
    }
    fn energy_count(&mut self, slot: SlotRef) -> R<i32> {
        let map = crate::engine::attack::provided_energy_read(self, slot.p as usize, slot)?;
        Ok(crate::engine::attack::max_energy_count(&map))
    }
    fn payable(&mut self, attack: AttackRef, slot: SlotRef) -> R<bool> {
        crate::engine::attack::attack_payable(self, slot.p as usize, attack, slot)
    }
}

/// Every check of player `p` using `attack`, in order (`ignore_status`: the Special Conditions don't stop it;
/// `granted_first_turn`: an Ability lets it be used on the first turn, Meloetta ex; execution passes `false`, the flag is
/// written by then). Returns the attacking spot, or the code of the refusal (the action is illegal). An error of a read
/// is propagated.
pub fn attack_checks<C: ActionChecks + ?Sized>(c: &mut C, p: usize, attack: AttackRef, ignore_status: bool, granted_first_turn: bool) -> R<Result<SlotRef, &'static str>> {
    let attacking = match crate::engine::attack::can_attack_pre(c.game(), p, attack, ignore_status, granted_first_turn) {
        Ok(s) => s,
        Err(e) => return Ok(Err(e.0)),
    };
    let max_energy = if crate::engine::attack::attack_max_energy_applies(c.game(), p) { Some(c.energy_count(attacking)?) } else { None };
    if let Err(e) = crate::engine::attack::can_attack_post(c.game(), p, attack, attacking, max_energy) {
        return Ok(Err(e.0));
    }
    let v = attack_view(c.game(), p, attacking);
    if let Some(code) = c.event_locked(&v)? {
        return Ok(Err(code));
    }
    if !c.payable(attack, attacking)? {
        return Ok(Err("NOT_ENOUGH_ENERGY"));
    }
    Ok(Ok(attacking))
}

/// The locks over player `p` using the Ability `power` of `card` (the UseAbility event; no pool card prints "can't use
/// Abilities": an Ability lock means the Ability isn't there, `derived::has_no_ability`).
pub fn ability_locked<C: ActionChecks + ?Sized>(c: &mut C, p: usize, power: PowerRef, card: CardId) -> R<Option<&'static str>> {
    let v = ability_view(c.game(), p, power, card);
    c.event_locked(&v)
}

/// Every check of player `p` using the Stadium in play (APR B-04): once during each player's turn, a Stadium that
/// declares a use, the locks over UseStadium. Returns the Stadium or the refusal.
pub fn stadium_checks<C: ActionChecks + ?Sized>(c: &mut C, p: usize) -> R<Result<CardId, &'static str>> {
    let stadium = match crate::engine::turn::can_use_stadium(c.game(), p) {
        Ok(s) => s,
        Err(e) => return Ok(Err(e.0)),
    };
    let v = stadium_view(c.game(), p, stadium);
    if let Some(code) = c.event_locked(&v)? {
        return Ok(Err(code));
    }
    Ok(Ok(stadium))
}
