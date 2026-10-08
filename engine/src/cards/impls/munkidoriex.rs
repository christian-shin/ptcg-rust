//! Munkidori ex (SFA): Oh No You Don't — if this Pokémon is Knocked Out and
//! you have Pecharunt ex in play, the opponent takes 1 fewer Prize card.
//! Dirty Headbutt — 190; during your next turn this Pokémon can't use it.
//!
//! Twinleaf (phase 4b): a KnockOutEffect on this card's slot counts only
//! during the opponent's ATTACK phase with the owner carrying
//! DAMAGE_DEALT_MARKER (Knocked Out by damage from an attack; it used to count
//! any KO, e.g. Poison), and "Pecharunt ex in play" is a scan of the owner's
//! Pokémon (it used the owner's `pecharuntexIsInPlay` flag, which Pecharunt ex
//! set only while its owner's Active had a Special Condition and never
//! cleared).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Munkidoriex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotUseThisAttackNextTurn }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Knocked Out by damage from an attack with Pecharunt ex in play: the opponent takes 1 fewer Prize card.
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec {
            delta: -1,
            subject: SlotPred::Holder,
            by_attack_damage: true,
            by_own_attack: None,
            guard: Cond::AnySlot(Who::Me, SlotPred::Named("Pecharunt ex")),
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
