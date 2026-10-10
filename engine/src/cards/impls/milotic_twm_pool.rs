//! Milotic (TWM 50): Mentally Calm — your opponent's Pokémon in play and all
//! attached cards can't be put into your opponent's hand. Hydro Splash — 100.
//!
//! Mentally Calm is a `Prevent` over the opponent's Pokémon's LeavePlay into their hand (events batch 7, user decision D1:
//! a whole Pokémon, and every card attached to one, leaving play is a LeavePlay whose destination is a consequence; the
//! hand is always the owner's, APR C-02). Whatever the cause: the opponent's own effects too.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MiloticTWMPool",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Prevent(PreventSpec::on(SlotPred::Not(&SlotPred::OnMySide), EventPred::All(&[EventPred::Kind(EventKind::LeavePlay), EventPred::Dest(RulesZone::Hand)]))),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
