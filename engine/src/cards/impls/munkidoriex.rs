//! Munkidori ex (SFA 37): Oh No You Don't — if this Pokémon is Knocked Out by
//! damage from an attack from your opponent's Pokémon, and if you have any
//! Pecharunt ex in play, your opponent takes 1 fewer Prize card. Dirty Headbutt —
//! 190; during your next turn, this Pokémon can't use Dirty Headbutt.
//!
//! Rule: a `PrizeAdjust` over the KnockOut view with `ko_by` AttackDamage (Poison
//! or an effect that Knocks it Out doesn't count) and the guard "Pecharunt ex in
//! play" (a scan of your Pokémon when the Prizes are taken). Dirty Headbutt arms
//! `CannotUseThisAttackNextTurn` (one ApplyEffect event on this Pokémon).
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
            guard: Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::Named("Pecharunt ex")),
            ..PrizeAdjustSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
