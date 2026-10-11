//! Milotic (TWM 50): Mentally Calm — your opponent's Pokémon in play and all
//! attached cards can't be put into your opponent's hand. Hydro Splash — 100.
//!
//! Mentally Calm is a lock over the LeavePlay into the hand of the opponent's Pokémon and of every card attached to one
//! (events batch 7, user decision D1: a whole Pokémon, and every card attached to one, leaving play is a LeavePlay whose
//! destination is a consequence; the hand is always the owner's, APR C-02). A flat "can't be put" is a lock, binding
//! whoever would put them (both players: the opponent's own effects too), like Poké Vital A's "can't be put into your hand
//! or deck" (RULES.md, "one rule per phrase"); a `Prevent` is "prevent effects ... done to".
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MiloticTWMPool",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::BlockUse(BlockUseSpec {
            binds: Binds::Both,
            lock: LockDecl::on(EventPred::All(&[EventPred::Kind(EventKind::LeavePlay), EventPred::Slot(SlotPred::Not(&SlotPred::OnMySide)), EventPred::Dest(RulesZone::Hand)]), "BLOCKED_BY_ABILITY"),
            while_: &[],
            ability: true,
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
