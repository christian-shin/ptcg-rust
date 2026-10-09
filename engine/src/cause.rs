//! `Cause`: what made a rules event happen (docs/design/events-design.md, section 3).
//!
//! Every event that can be prevented, or that cards react to, carries one. "Effect of your opponent's
//! attack", "your opponent's Abilities", "by an effect of an attack" are predicates over it (plus the
//! turn). The interpreter fills it once per program frame (`spec::run::Frame::cause`); the core rule
//! routines fill it with their rule (turn draw, retreat, promotion, Checkup, setup). Ops never pick a
//! cause per card.
//!
//! Events batch 1: the field is filled everywhere, but the old inference (`AtkBase` presence, the
//! Hide 'n' Sneak kind list, op flags like `CounterCause` / `effect_of_attack`) is still what the
//! readers use. `PTCG_VERIFY_CACHE=1` compares the two and counts the disagreements per class
//! (`mismatch`, `report`).

use crate::list::CardId;
use crate::state::AttackRef;

/// Which game rule caused an event that no card did.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum RuleWhich {
    /// The draw at the start of the turn.
    TurnDraw,
    Retreat,
    /// Promoting a Benched Pokémon after the Active Pokémon left play.
    Promotion,
    /// Pokémon Checkup (between turns).
    Checkup,
    /// Setting up the game (placing the Active and Benched Pokémon, Prizes, mulligans).
    Setup,
    /// A player's own action under the game rules during their turn: playing a Pokémon from the hand onto
    /// the Bench, evolving from the hand, the turn's Energy attachment, playing a Trainer card.
    /// (Not in the design's list; added in batch 1, see the batch report.)
    Action,
    /// The end of the turn (the game's end-of-turn bookkeeping of a card's own text).
    /// (Not in the design's list; added in batch 1, see the batch report.)
    EndTurn,
    /// A rule printed on a card that is not an Ability (the Tera rule, a Fossil's rules), outside the
    /// end of the turn. (Not in the design's list; added in batch 2.)
    CardRule,
}

/// The kind of thing that caused an event.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum CauseKind {
    Attack,
    Ability,
    /// An Item or Supporter card's effect.
    Trainer,
    Stadium,
    Tool,
    Energy,
    Rule { which: RuleWhich },
    /// Poison, Burn, Confusion, Sleep, Paralysis.
    SpecialCondition,
}

/// What caused an event. `Copy`, 7 bytes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cause {
    pub kind: CauseKind,
    /// The card whose attack, Ability, Trainer, Stadium, Tool or Energy it is (the attacking Pokémon for
    /// an attack, also when the attack is copied).
    pub card: Option<CardId>,
    /// The player who caused it (the user of the attack / Ability / Trainer, the owner of the card for a
    /// Tool, Energy, Stadium or a triggered Ability; the player the rule acts for).
    pub player: u8,
    /// The attack, for `CauseKind::Attack`.
    pub attack: Option<AttackRef>,
}

impl Cause {
    pub const fn new(kind: CauseKind, card: Option<CardId>, player: u8) -> Cause {
        Cause { kind, card, player, attack: None }
    }

    /// An effect of `attack`, used by `player`'s Pokémon `card`.
    pub const fn attack(player: u8, card: Option<CardId>, attack: AttackRef) -> Cause {
        Cause { kind: CauseKind::Attack, card, player, attack: Some(attack) }
    }

    /// A game rule acting for `player`.
    pub const fn rule(which: RuleWhich, player: u8) -> Cause {
        Cause { kind: CauseKind::Rule { which }, card: None, player, attack: None }
    }

    /// The cause in 4 bytes, for a program frame that must carry it across a prompt (`CardFrame` has no room for
    /// the 7-byte struct): kind and rule (6 bits) and player (1 bit), the card, the attack's card and index
    /// (`0xFF` = none; a game has fewer card instances).
    pub const fn pack(&self) -> [u8; 4] {
        let (k, w) = match self.kind {
            CauseKind::Attack => (0u8, 0u8),
            CauseKind::Ability => (1, 0),
            CauseKind::Trainer => (2, 0),
            CauseKind::Stadium => (3, 0),
            CauseKind::Tool => (4, 0),
            CauseKind::Energy => (5, 0),
            CauseKind::Rule { which } => (6, which as u8),
            CauseKind::SpecialCondition => (7, 0),
        };
        let card = match self.card {
            Some(c) => c,
            None => 0xFF,
        };
        let (ac, ai) = match self.attack {
            Some(a) => (a.card, a.index),
            None => (0xFF, 0xFF),
        };
        [k | w << 3 | (self.player & 1) << 7, card, ac, ai]
    }

    /// The inverse of [`Cause::pack`].
    pub const fn unpack(b: [u8; 4]) -> Cause {
        const WHICH: [RuleWhich; 8] = [
            RuleWhich::TurnDraw,
            RuleWhich::Retreat,
            RuleWhich::Promotion,
            RuleWhich::Checkup,
            RuleWhich::Setup,
            RuleWhich::Action,
            RuleWhich::EndTurn,
            RuleWhich::CardRule,
        ];
        let kind = match b[0] & 7 {
            0 => CauseKind::Attack,
            1 => CauseKind::Ability,
            2 => CauseKind::Trainer,
            3 => CauseKind::Stadium,
            4 => CauseKind::Tool,
            5 => CauseKind::Energy,
            6 => CauseKind::Rule { which: WHICH[((b[0] >> 3) & 7) as usize] },
            _ => CauseKind::SpecialCondition,
        };
        Cause {
            kind,
            card: if b[1] == 0xFF { None } else { Some(b[1]) },
            player: b[0] >> 7,
            attack: if b[2] == 0xFF && b[3] == 0xFF { None } else { Some(AttackRef { card: b[2], index: b[3] }) },
        }
    }

    pub const fn is_attack(&self) -> bool {
        matches!(self.kind, CauseKind::Attack)
    }

    pub const fn is_ability(&self) -> bool {
        matches!(self.kind, CauseKind::Ability)
    }

    /// The effect of an attack of an `AttackEffect` (`Effect::Attack` / `AfterAttack`): the attacking
    /// player, the Pokémon in the attacking slot.
    pub fn of_attack_effect(g: &crate::game::Game, atk: crate::effects::EffId) -> Option<Cause> {
        if atk as usize >= g.fx.len() {
            return None;
        }
        let (p, _, attack, source) = crate::prefabs::attack_data(g, atk)?;
        Some(Cause::of_attack_at(g, p, attack, source))
    }

    /// An effect of `attack` used by `p` from the slot `source`: the cause's card is the Pokémon there
    /// (the attack's own card when the slot is empty).
    pub fn of_attack_at(g: &crate::game::Game, p: u8, attack: AttackRef, source: crate::effects::SlotRef) -> Cause {
        let card = g.st.slot_pokemon(source.p as usize, source.s).or(Some(attack.card));
        Cause::attack(p, card, attack)
    }

    /// The cause of a card's own rule text by its origin (a passive's `RuleSource`). `CardRule` (a rule
    /// printed on the card that is not an Ability) is `Rule { CardRule }`; a trigger's cause also depends
    /// on its event (`spec::run::frame_cause`).
    pub fn of_origin(origin: crate::spec::passive::RuleSource, card: CardId, player: u8) -> Cause {
        use crate::spec::passive::RuleSource;
        let kind = match origin {
            RuleSource::Ability => CauseKind::Ability,
            RuleSource::Tool => CauseKind::Tool,
            RuleSource::Energy => CauseKind::Energy,
            RuleSource::Stadium => CauseKind::Stadium,
            RuleSource::TrainerEffect => CauseKind::Trainer,
            RuleSource::CardRule => CauseKind::Rule { which: RuleWhich::CardRule },
        };
        Cause::new(kind, Some(card), player)
    }

    /// A Trainer card's own kind as a cause: an Item or Supporter is `Trainer`, a Tool `Tool`, a Stadium
    /// `Stadium`.
    pub fn of_trainer(g: &crate::game::Game, card: CardId, player: u8) -> Cause {
        use crate::types::TrainerType;
        let d = g.st.cdef(card);
        let kind = if d.trainer_type == TrainerType::Tool as u8 {
            CauseKind::Tool
        } else if d.trainer_type == TrainerType::Stadium as u8 {
            CauseKind::Stadium
        } else {
            CauseKind::Trainer
        };
        Cause::new(kind, Some(card), player)
    }
}

// ---------------------------------------------------------------------------
// VERIFY: the new cause against the old inference.

static MISMATCHES: std::sync::Mutex<std::collections::BTreeMap<String, u64>> = std::sync::Mutex::new(std::collections::BTreeMap::new());
static CHECKS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Is the cross-check on (`PTCG_VERIFY_CACHE=1` or `PTCG_VERIFY_LEGAL=1`)?
#[inline]
pub fn verify() -> bool {
    crate::game::verify_cache()
}

/// One comparison was made.
pub fn checked() {
    CHECKS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// Count a disagreement of class `class` (built only when VERIFY is on).
pub fn mismatch(class: String) {
    *MISMATCHES.lock().unwrap().entry(class).or_insert(0) += 1;
}

/// What an old inference says about an event's cause.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Old {
    /// An effect of an attack (by this player, when it says).
    Attack(Option<u8>),
    /// An effect of an Ability (by this player, when it says).
    Ability(Option<u8>),
    /// Not an effect of an attack (it doesn't decide anything else).
    NotAttack,
    /// Neither an attack's nor an Ability's.
    Neither,
}

/// Is the cross-check on for this game (VERIFY, and not a legality trial)?
#[inline]
pub fn on(g: &crate::game::Game) -> bool {
    verify() && !g.trial
}

/// Compare an old inference with the new cause; `site` names the inference (no-op unless `on`).
#[inline]
pub fn compare(g: &crate::game::Game, site: &str, c: &Cause, old: Old) {
    if on(g) {
        compare_now(g, site, c, old);
    }
}

#[cold]
fn compare_now(g: &crate::game::Game, site: &str, c: &Cause, old: Old) {
    checked();
    let ok = match old {
        Old::Attack(p) => c.is_attack() && p.map_or(true, |p| p == c.player),
        Old::Ability(p) => c.is_ability() && p.map_or(true, |p| p == c.player),
        Old::NotAttack => !c.is_attack(),
        Old::Neither => !c.is_attack() && !c.is_ability(),
    };
    if !ok {
        let player_note = match old {
            Old::Attack(Some(p)) | Old::Ability(Some(p)) if p != c.player => " (other player)",
            _ => "",
        };
        let old_s = match old {
            Old::Attack(_) => "attack",
            Old::Ability(_) => "Ability",
            Old::NotAttack => "not an attack",
            Old::Neither => "neither attack nor Ability",
        };
        mismatch(format!("{site}: old {old_s}{player_note}, cause {}", label(g, c)));
    }
}

/// The cause for the mismatch classes (no player: the classes group across seats).
fn label(g: &crate::game::Game, c: &Cause) -> String {
    let card = c.card.map(|x| g.st.cdef(x).full_name).unwrap_or("-");
    format!("{:?} ({})", c.kind, card)
}

/// An event the old code carries out without an effect anyone can see (a direct write): counted by its cause.
#[inline]
pub fn unseen(g: &crate::game::Game, site: &str, c: &Cause) {
    if on(g) {
        unseen_now(g, site, c);
    }
}

#[cold]
fn unseen_now(g: &crate::game::Game, site: &str, c: &Cause) {
    checked();
    mismatch(format!("{site}: no event, cause {}", label(g, c)));
}

/// The old inferences that read an effect's kind and fields, against the effect's cause (called for every
/// effect created outside legality trials when VERIFY is on):
/// - the `AtkBase` makes an effect "an effect of the attack by `b.player`" (Hide 'n' Sneak, Mist Energy,
///   "prevent all effects of attacks", the attack reducer's own check);
/// - Hide 'n' Sneak's Ability half: an `AddSpecialConditionsPower` whose source is a Pokémon in play is an
///   Ability of that Pokémon's side; a `PlaceDamageCounters` whose source is a Pokémon in play of `p` is
///   `p`'s Ability; `EffectOfAbility` is an Ability of `p`;
/// - an opponent's attack or Ability effect on a Pokémon whose kind is missing from
///   `HIDE_N_SNEAK_KINDS` escapes Hide 'n' Sneak.
#[inline]
pub fn verify_effect(g: &crate::game::Game, e: &crate::effects::Effect) {
    if on(g) {
        verify_effect_now(g, e);
    }
}

#[cold]
fn verify_effect_now(g: &crate::game::Game, e: &crate::effects::Effect) {
    use crate::effects::Effect;
    let Some(c) = e.cause() else { return };
    if let Some(b) = e.atk_base() {
        compare_now(g, &format!("{} (AtkBase = attack)", e.type_name()), &c, Old::Attack(Some(b.player)));
        return;
    }
    let target = match *e {
        Effect::AddSpecialConditionsPower { source, target: _, .. } => {
            let side = if g.st.cdef(source).is_pokemon() { g.st.find_pokemon_slot(source).map(|(q, _)| q as u8) } else { None };
            let old = match side {
                Some(q) => Old::Ability(Some(q)),
                None => Old::Neither,
            };
            compare_now(g, "ADD_SPECIAL_CONDITIONS_POWER (source Pokemon in play = Ability)", &c, old);
            None
        }
        Effect::PlaceDamageCounters { p, source, .. } => {
            let ability = source != crate::list::NO_CARD && matches!(g.st.find_pokemon_slot(source), Some((q, _)) if q == p as usize);
            let old = if ability { Old::Ability(Some(p)) } else { Old::Neither };
            compare_now(g, "PLACE_DAMAGE_COUNTERS (source Pokemon of p in play = Ability)", &c, old);
            None
        }
        Effect::EffectOfAbility { p, target, .. } => {
            compare_now(g, "EFFECT_OF_ABILITY (probe = Ability)", &c, Old::Ability(Some(p)));
            target
        }
        Effect::Heal { target, .. } | Effect::Evolve { target, .. } | Effect::Attach { target, .. } => Some(target),
        // ChangeActive asks the `Prevent` reader itself (events batch 5): no kind list to escape.
        _ => None,
    };
    if let Some(t) = target {
        if (c.is_attack() || c.is_ability()) && t.p != c.player && !crate::spec::passive::HIDE_N_SNEAK_KINDS.contains(&e.kind()) {
            checked();
            mismatch(format!("{} on the opponent's Pokemon, kind not in HIDE_N_SNEAK_KINDS: cause {}", e.type_name(), label(g, &c)));
        }
    }
}

/// The comparisons made and the disagreements by class, for the end of a VERIFY run.
pub fn report() -> String {
    let m = MISMATCHES.lock().unwrap();
    let total: u64 = m.values().sum();
    let mut s = format!("cause cross-check: {} comparisons, {} mismatches in {} classes\n", CHECKS.load(std::sync::atomic::Ordering::Relaxed), total, m.len());
    for (k, v) in m.iter() {
        s.push_str(&format!("  {v:>8}  {k}\n"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cause_is_small_and_copy() {
        assert!(std::mem::size_of::<Cause>() <= 8, "Cause is {} bytes", std::mem::size_of::<Cause>());
        let c = Cause::rule(RuleWhich::Checkup, 1);
        let d = c;
        assert_eq!(c, d);
    }

    #[test]
    fn pack_round_trips() {
        let whiches = [RuleWhich::TurnDraw, RuleWhich::Retreat, RuleWhich::Promotion, RuleWhich::Checkup, RuleWhich::Setup, RuleWhich::Action, RuleWhich::EndTurn, RuleWhich::CardRule];
        let mut all = vec![
            Cause::attack(1, Some(17), AttackRef { card: 17, index: 1 }),
            Cause::attack(0, None, AttackRef { card: 3, index: 0x90 }),
            Cause::new(CauseKind::Ability, Some(0), 0),
            Cause::new(CauseKind::Trainer, Some(119), 1),
            Cause::new(CauseKind::Stadium, Some(5), 0),
            Cause::new(CauseKind::Tool, Some(6), 1),
            Cause::new(CauseKind::Energy, Some(7), 0),
            Cause::new(CauseKind::SpecialCondition, None, 1),
        ];
        for w in whiches {
            all.push(Cause::rule(w, 1));
        }
        for c in all {
            assert_eq!(Cause::unpack(c.pack()), c);
        }
    }
}
